# Protocole du démon MIRA

> **Statut :** décrit ce que le code de `mira-protocol` et `mira-daemon` fait à la fin de la phase 2 (spec : `docs/superpowers/specs/2026-11-04-architecture-demon-design.md`). C'est la source écrite que le client Dart réimplémentera en phase 3. Le contrat d'API (`CONTRACT_VERSION`) n'est pas modifié : seul le transport est nouveau.

## 1. Transports

Un transport ne fournit qu'un **flux d'octets bidirectionnel fiable**. Le démon en propose deux :

| Transport | Option | Protection |
|---|---|---|
| Socket Unix | défaut (`$XDG_RUNTIME_DIR/mira-daemon/<profil>.sock`, sinon `<dossier du profil>/daemon.sock`, ou `--socket`) | dossier `0700`, socket `0600`, uid du pair égal à celui du démon, plus le jeton |
| TCP loopback | `--tcp-port <n>` (`0` = port libre, annoncé sur stdout) | écoute sur `127.0.0.1` seulement, jeton |

Le démon n'écoute jamais ailleurs que sur le loopback. Il n'y a pas de named pipe (Windows : TCP loopback, phase 5) ni de TLS (version 4).

## 2. Trames

Une trame est un corps **JSON** précédé de sa longueur sur **4 octets, grand-boutiste**. La longueur annoncée est comparée au maximum du démon (16 Mio par défaut) **avant** toute lecture du corps : une trame trop grande, vide ou illisible ferme la connexion. Chaque trame est un objet avec un champ `type` ; les échanges portent un `id` (entier non signé 64 bits) choisi par le client.

| `type` | Sens | Champs (en plus de `type`) |
|---|---|---|
| `hello` | C→S | `protocol_version {major, minor}`, `contract_version {major, minor}`, `client_id`, `client_name`, `auth` |
| `hello_ok` | S→C | `protocol_version`, `contract_version`, `host_state` |
| `hello_err` | S→C | `reason` : `protocol_version`, `contract_version`, `auth` ou `malformed` (aucun texte libre ; `malformed` pour un `hello` illisible ou un moyen d'authentification inconnu) |
| `call` | C→S | `id`, `request` (la requête du contrat, objet JSON) |
| `result` | S→C | `id`, `envelope` (l'enveloppe du contrat : `{contract_version, ok, data \| error}`) |
| `open_events` | C→S | `id`, `after` (entier ou `null`), `credit` (fenêtre initiale) |
| `open_chat` | C→S | `id`, `request` (le `ChatStart` du contrat), `credit` |
| `get_original_html` | C→S | `id`, `message` (identifiant du message) ; la réponse est un `result` |
| `host_op` | C→S | `id`, `op` : `shutdown` ou `retry_unlock` (protocole 1.1) ; la réponse est un `result` |
| `set_presence` | C→S | `id`, `focused` (booléen : la fenêtre du client est au premier plan) ; la réponse est un `result` (protocole 1.1). Chaque connexion compte pour une présence ; sa fermeture la retire. Le démon notifie ou non selon qu'un client est au premier plan |
| `event` | S→C | `id`, `event` (un événement du journal) |
| `chat_delta` | S→C | `id`, `line` (une ligne du flux de chat : `started`, `text`, `usage`, `done`, `error`, …) |
| `credit` | C→S | `id`, `n` (trames de plus autorisées pour ce flux) |
| `cancel` | C→S | `id` |
| `end` | S→C | `id`, `reason` : `{"type":"finished"}`, `{"type":"cancelled"}`, `{"type":"lagged"}` ou `{"type":"error","code":…,"message":…}` (cette dernière raison est réservée : le démon actuel ne l'émet pas) |
| `ping` / `pong` | C→S / S→C (le démon répond `pong` au `ping` du client, avec le même `nonce` ; il n'envoie jamais de `ping` ; un `pong` reçu du client est ignoré) | `nonce` |

`auth` est un objet typé : `{"type":"local_token","token":"<64 caractères hexadécimaux>"}`. `host_state` vaut `ready`, `starting`, `keyring_locked` ou `update_required` ; le démon annonce `starting` (moteur en cours d'ouverture), `ready` et `keyring_locked` (protocole 1.1 : le trousseau refuse l'accès parce qu'il est verrouillé ; le démon réessaie après 5 s, puis 10 s, 20 s… jusqu'à 60 s, et aussitôt sur `retry_unlock`, puis passe à `starting` et `ready` sans intervention). `update_required` est annoncé à un client dont le contrat est plus récent que celui du moteur (le moteur est périmé) : le démon répond `hello_ok` avec cet état au lieu de `hello_err contract_version`, puis n'accepte plus que `host_op shutdown`, `ping` et `pong` (toute autre trame ferme la connexion). Le jeton reste vérifié avant les versions ; une autre majeure du protocole ou un contrat d'une majeure plus ancienne reste refusé.

## 3. Poignée de main

1. Le client envoie `hello` **dans les 5 secondes** qui suivent la connexion (la lecture de la trame entière est bornée, pas octet par octet), et sa taille ne dépasse pas **8 Kio** (jugée sur l'en-tête). Au plus 64 connexions attendent leur `hello` en même temps ; les suivantes sont fermées aussitôt. Toute autre première trame, le silence ou la lenteur ferment la connexion sans réponse.
2. Le démon vérifie le **jeton d'abord** (comparaison en temps constant), puis les versions : majeure du protocole égale ; version du contrat compatible selon la règle de `mira-vocabulary` (même majeure, mineure du client inférieure ou égale à celle du moteur).
3. Succès : `hello_ok`. Échec : `hello_err` à raison fixe, puis fermeture. Le jeton n'est jamais renvoyé, journalisé ni écrit sur stdout ou stderr.
4. Un second `hello` ferme la connexion.

Le jeton est un fichier de 32 octets aléatoires en hexadécimal, créé en `0600` (`--token-file`, défaut `<dossier du profil>/daemon.token`). Un fichier lisible par le groupe ou les autres est refusé.

## 4. Appels

`call` est exécuté par le moteur dans une tâche indépendante : plusieurs appels peuvent être en vol sur une connexion, les réponses arrivent dans n'importe quel ordre et s'apparient par `id`. Une requête illisible reçoit une enveloppe d'échec, pas une fermeture. Une réponse plus grande que la taille maximale d'une trame devient une enveloppe d'échec au message fixe `réponse trop grande`. Si l'écriture vers le client échoue, la session se termine (les appels déjà lancés continuent). **Fermer la connexion n'annule jamais un appel** : un envoi de message ou une synchronisation lancés par un client se terminent même s'il disparaît.

Un `id` ne peut pas être réutilisé tant que son échange est en cours ; le faire ferme la connexion. `credit` et `cancel` sur un `id` inconnu (course avec un `end`) sont ignorés, de même que `cancel` sur un appel (un appel n'est pas annulable). Une trame **d'un type inconnu** reçue après `hello` (client plus récent) est ignorée : la connexion continue, mais l'échange attendu n'aura pas de réponse. Si le client ferme seulement son côté écriture, le démon laisse une seconde pour écrire les réponses en file avant de fermer.

## 5. Flux

`open_events` et `open_chat` ouvrent un flux sous un `id`. Le client donne une **fenêtre de crédits** : chaque `event` ou `chat_delta` en consomme un, `credit` en ajoute.

- **Événements.** Sans `after`, seuls les événements futurs sont livrés (le journal passé n'est pas rejoué) ; avec `after: n`, le journal est rejoué à partir de `n` (exclu). Le démon garde une file de 256 événements par flux. Si elle reste pleine plus de 5 secondes (le client ne lit plus ou n'a plus de crédit), le flux se termine par `end` `lagged` : le client se réabonne avec `after` égal au `seq` du dernier événement reçu, sans perte ni doublon.
- **Chat.** Chaque ligne attend un crédit : sans crédit, la lecture du fournisseur est suspendue, pas abandonnée. `cancel`, ou la fermeture de la connexion, annule l'opération (le fournisseur voit sa connexion fermée, l'opération passe à `cancelled`) ; le flux se termine par `end` `cancelled`. L'annulation est constatée à la ligne suivante, comme pour l'ancien pont.
- Un flux terminé normalement envoie `end` `finished`. Sur un moteur jamais ouvert, un chat reçoit une seule ligne `error` au message fixe puis `end`.

## 6. Opérations d'hôte

`host_op` `shutdown` répond `result` (succès) puis arrête le démon (un seul arrêt à la fois, même demandé deux fois ou par un signal en même temps) : il cesse d'accepter, ferme les connexions (une seconde pour écrire ce qui est en file), laisse finir les appels en vol (5 secondes au plus), ferme le moteur et libère le verrou du profil. Toute connexion authentifiée peut l'envoyer. `retry_unlock` (protocole 1.1) réveille l'attente du trousseau verrouillé (§2, `host_state`). Le relevé n'a pas d'opération d'hôte : il se règle par `set_settings.poll_secs`, appliqué sans redémarrage.

## 7. Le démon

`mira-daemon [options]` : `--profile`, `--home`, `--poll-secs`, `--secrets-key-file` (32 octets, `0600`, remplace le trousseau du système par un fichier chiffré), `--socket`, `--tcp-port`, `--token-file`, `--config-file`, `--exit-when-idle` et `--idle-grace-secs` (30 par défaut). Le fichier `daemon.toml` du dossier MIRA (à côté de `profiles/`) accepte `profile`, `home`, `socket` et `token_file` ; une option de la ligne de commande l'emporte, une clé inconnue ou un fichier de plus de 64 Kio est refusé (code 2, message fixe). Le comportement (relevé, heures calmes, sourdine, notifications) n'est pas dans ce fichier : il vit dans les réglages du moteur.

**À la demande ou résident.** Lancé avec `--exit-when-idle` (par l'app), le démon s'arrête quand plus aucun client n'est resté attaché pendant le délai de grâce, la grâce courant dès qu'il est prêt. Lancé sans (unité `systemd --user`, `Type=simple`, `Restart=on-failure`), il reste actif.

**Trousseau verrouillé.** Avant d'ouvrir le moteur, le démon demande au Secret Service (D-Bus `org.freedesktop.secrets`, propriété `Locked` de la collection par défaut, sans invite et sans réveiller un service arrêté) si le trousseau est verrouillé ; avec `--secrets-key-file` il n'y a rien à déverrouiller. S'il l'est, il annonce `keyring_locked`, envoie une notification unique, réessaie après 5 s puis 10 s, 20 s… jusqu'à 60 s, et aussitôt sur `retry_unlock`, puis ouvre le moteur.

**Notifications et icône.** Après `ready`, le démon écoute les arrivées du moteur et affiche des notifications du bureau par D-Bus (`org.freedesktop.Notifications`, action « Ouvrir » qui relance l'app) selon les réglages du moteur et la présence des clients (`set_presence`). Il affiche une icône de zone de notification (StatusNotifierItem) : « Ouvrir MIRA », une ligne d'état, « Quitter MIRA ». Sans bus de session ou sans hôte de zone de notification, il le dit une fois sur sa sortie et continue. `MIRA_APP_BIN` désigne l'app à relancer ; sinon le binaire `mira` voisin, sinon `gtk-launch io.github.nnoah29.Mira`.

Il écoute d'abord (les clients voient `starting`), ouvre ensuite le moteur, écrit `tcp_port=<n>` (si TCP) dès que l'écouteur est lié, puis `mira-daemon prêt` quand le moteur est ouvert. SIGTERM et SIGINT déclenchent le même arrêt que `shutdown`.

| Code de sortie | Sens |
|---|---|
| 0 | arrêt normal |
| 1 | échec (socket ou port inutilisable, ouverture du moteur impossible) |
| 2 | configuration invalide (chemins, jeton, fichier de clé) |
| 3 | un autre démon tient déjà la socket, ou le profil est déjà tenu par un autre moteur (D-27) — un démon, mais aussi le moteur embarqué d'une app |

Une socket laissée par un démon tué est remplacée au redémarrage (seulement si la connexion est *refusée* : une socket qu'on ne peut pas sonder n'est jamais supprimée) ; une socket sur laquelle quelqu'un écoute n'est jamais volée. Un verrou `<socket>.lock` empêche deux démons qui démarrent en même temps de se disputer la socket, et le dossier de la socket doit appartenir à l'utilisateur du démon.

## 8. Sécurité

Aucune trame émise par le démon ne contient un secret (mots de passe, clés d'IA, jetons OAuth, jeton local) ; seul `has_key` dit qu'une clé existe. Les secrets que le contrat accepte déjà **en entrée** (mot de passe de `AddAccount`, clé de `SetAiApiKey`) restent permis dans les `call` entrants : ces transports sont locaux et authentifiés. Le test `secrets_in_frames` capture les octets émis par le démon pendant un scénario complet.

**Limite connue du TCP loopback.** Le jeton est envoyé en clair dans `hello` : un autre utilisateur local qui occuperait avant le démon le port TCP fixe d'un client le récolterait. Un défi-réponse n'y changerait presque rien (le pirate peut relayer le défi vers le vrai démon) ; la vraie protection est le transport authentifié de la version 4 (TLS, appairage). D'ici là, le client Dart n'utilise que la socket Unix (dossier privé, uid du pair vérifié) et le TCP loopback est réservé aux plateformes sans socket Unix (Windows, phase 5), à utiliser en connaissance de cause.

## 9. Client Dart

Le client vit dans `app/lib/engine/remote/` ; `RemoteEngineClient` implémente `EngineClient` et `ChatStreamingClient`, comme `RustEngineClient` (pont FFI).

| Fichier | Rôle |
|---|---|
| `frames.dart`, `frame_builders.dart` | codec incrémental des trames et constructeurs des trames clientes ; les octets sont vérifiés contre les vecteurs `engine/crates/mira-protocol/tests/golden/frames.json`, partagés avec le test Rust `golden` |
| `byte_channel.dart` | interface `ByteChannel` (flux d'octets) et `SocketChannel` (socket Unix ou TCP loopback) |
| `connection.dart` | poignée de main, appariement des `result` et des flux par `id`, `ping` toutes les 15 s, connexion déclarée morte au 4ᵉ intervalle sans aucune trame reçue (donc entre 45 et 60 s de silence) |
| `remote_engine_client.dart` | appels, HTML d'origine, détachement (`close`), arrêt du démon (`shutdown`), reconnexion |
| `event_stream.dart`, `chat_stream.dart` | flux d'événements et de chat avec crédits |
| `daemon_paths.dart`, `daemon_token.dart`, `daemon_launcher.dart` | chemins (mêmes règles que le moteur), lecture du jeton (refusé s'il est lisible par d'autres), lancement à la demande de `mira-daemon` (processus surveillé), redémarrage d'un démon périmé, mode résident (`resident.dart`) |
| `engine_opening.dart` | choix démon / embarqué et états de l'hôte |

Règles du client :

- **Crédits.** Événements : fenêtre de 256, 128 rendus tous les 128 événements remis. Chat : fenêtre de 64, 32 rendus toutes les 32 lignes. Tant que l'abonné est en pause, les crédits sont retenus (le démon cesse d'envoyer, la mémoire du client reste bornée) et rendus à la reprise.
- **Événements.** Après `end lagged`, le flux est rouvert aussitôt ; après `end finished` ou une coupure, il l'est avec une attente de 0,5 s doublée jusqu'à 5 s (remise à 0,5 s après un événement). Dans les deux cas `after` est le dernier `seq` reçu. `close()` du client termine le flux. Sans `after` donné, le client lit d'abord la fin actuelle du journal (appel `get_events` par pages de 500, 0 si le journal est vide) et ouvre le flux à partir de là : une coupure avant le premier événement ne perd donc rien. Une trame `event` ou `chat_delta` sans objet est ignorée (son crédit est compté).
- **Chat.** Annuler l'abonnement envoie `cancel`. Une coupure remet l'erreur `transport.disconnected` ; un chat n'est jamais rouvert.
- **Reconnexion.** Le jeton est relu à chaque reconnexion (le démon a pu redémarrer avec un autre) et un refus est rapporté précisément (`host.auth_refused`, `host.update_required`) au lieu de « injoignable ». Une requête de plus de 16 Mio est refusée localement (`transport.request_too_large`).
- **Appels.** Une coupure en plein appel remet `transport.disconnected` ; l'appel suivant se reconnecte (une seule connexion pour plusieurs appels simultanés), sinon `transport.unavailable`. Les trames de type inconnu envoyées par le démon sont ignorées.
- **Détachement.** `close()` ferme la connexion et n'arrête jamais le démon.
- **Choix du moteur** (`MIRA_ENGINE=auto|embedded|daemon`, défaut `auto`) : hors Linux de bureau, toujours le moteur embarqué. Sous Linux, `auto` s'attache à un démon vivant, sinon lance `mira-daemon` et l'attend 20 s (temps réel). Le binaire est cherché dans l'ordre : `MIRA_DAEMON_BIN` (une valeur qui désigne un fichier absent est une erreur, pas ignorée), `<dossier de l'exécutable>/mira-daemon`, `<dossier de l'exécutable>/lib/mira-daemon`, puis — hors version de production seulement — `engine/target/{release,debug}/mira-daemon` à la racine du dépôt retrouvée **depuis l'exécutable** ; jamais de chemin relatif au répertoire courant, sinon passe au moteur embarqué. `daemon` n'a pas ce repli. Les états `keyring_locked` et `update_required`, un jeton refusé ou illisible et une version incompatible sont des erreurs lisibles : **jamais de repli sur l'embarqué**, qui se heurterait au même verrou ou au même trousseau. Un hôte `starting` est interrogé toutes les 250 ms (30 s au plus, en temps réel). Pour `update_required` (le démon est plus ancien que l'app : il répond `hello_ok` avec cet état et n'accepte plus que `shutdown`), le client le redémarre une fois : `systemctl --user restart mira-daemon.service` si l'unité est active, sinon `host_op shutdown` puis relance à la demande ; s'il reste périmé 10 s après, erreur lisible. Un hôte `keyring_locked` reçoit `retry_unlock` à chaque essai pendant 20 s. Un démon lancé par l'app (`--exit-when-idle`, processus surveillé) qui s'arrête avant d'être prêt donne `transport.daemon_exited` avec la raison de son code (3 : profil déjà utilisé, 2 : configuration invalide) au lieu des 20 s d'attente.
- **Identité.** `client_id` : 128 bits aléatoires gardés dans les préférences (`engineClientId`) ; `client_name` : `mira-app`.

## 10. Où sont les tests

| Sujet | Test (`engine/crates/mira-daemon/tests/` sauf mention) |
|---|---|
| Trames et codec | `mira-protocol/tests/{frames,codec,handshake}.rs`, `wire.rs` |
| Jeton, transports | `token.rs`, `transport_{unix,unix_files,tcp}.rs` |
| Poignée de main, appels, limites | `session_{hello,hello_limits,calls,limits}.rs`, `exchanges.rs` |
| Flux, crédits, annulation | `stream_{events,chat,ids}.rs` |
| Arrêt, envois indépendants | `host_ops.rs`, `disconnect_send.rs` |
| Binaire | `config.rs`, `binary_lifecycle.rs` |
| Contrat sur trois chemins | `contract_direct.rs`, `contract_socket.rs`, `bridge/tests/contract_bridge.rs` (mêmes scénarios, `tests/contract/`) |
| Aucun secret dans les trames | `secrets_in_frames.rs` |
| Client Dart | `app/test/remote/` (trames, canaux, connexion, appels, événements, chat, chemins, jeton, lancement, politique de choix) ; `remote_daemon_test.dart` joue le vrai binaire (ignoré s'il n'est pas construit : `make daemon` ou `cd engine && cargo build -p mira-daemon`) |
