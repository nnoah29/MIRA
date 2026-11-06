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
| `hello_err` | S→C | `reason` : `protocol_version`, `contract_version`, `auth` ou `malformed` (aucun texte libre) |
| `call` | C→S | `id`, `request` (la requête du contrat, objet JSON) |
| `result` | S→C | `id`, `envelope` (l'enveloppe du contrat : `{contract_version, ok, data \| error}`) |
| `open_events` | C→S | `id`, `after` (entier ou `null`), `credit` (fenêtre initiale) |
| `open_chat` | C→S | `id`, `request` (le `ChatStart` du contrat), `credit` |
| `get_original_html` | C→S | `id`, `message` (identifiant du message) ; la réponse est un `result` |
| `host_op` | C→S | `id`, `op` : `shutdown` ; la réponse est un `result` |
| `event` | S→C | `id`, `event` (un événement du journal) |
| `chat_delta` | S→C | `id`, `line` (une ligne du flux de chat : `started`, `text`, `usage`, `done`, `error`, …) |
| `credit` | C→S | `id`, `n` (trames de plus autorisées pour ce flux) |
| `cancel` | C→S | `id` |
| `end` | S→C | `id`, `reason` : `{"type":"finished"}`, `{"type":"cancelled"}`, `{"type":"lagged"}` ou `{"type":"error","code":…,"message":…}` |
| `ping` / `pong` | les deux | `nonce` |

`auth` est un objet typé : `{"type":"local_token","token":"<64 caractères hexadécimaux>"}`. `host_state` vaut `ready`, `starting`, `keyring_locked` ou `update_required` ; le démon de la phase 2 n'annonce que `starting` (moteur en cours d'ouverture) et `ready`.

## 3. Poignée de main

1. Le client envoie `hello` **dans les 5 secondes** qui suivent la connexion (la lecture de la trame entière est bornée, pas octet par octet), et sa taille ne dépasse pas **8 Kio** (jugée sur l'en-tête). Au plus 64 connexions attendent leur `hello` en même temps ; les suivantes sont fermées aussitôt. Toute autre première trame, le silence ou la lenteur ferment la connexion sans réponse.
2. Le démon vérifie le **jeton d'abord** (comparaison en temps constant), puis les versions : majeure du protocole égale ; version du contrat compatible selon la règle de `mira-vocabulary` (même majeure, mineure du client inférieure ou égale à celle du moteur).
3. Succès : `hello_ok`. Échec : `hello_err` à raison fixe, puis fermeture. Le jeton n'est jamais renvoyé, journalisé ni écrit sur stdout ou stderr.
4. Un second `hello` ferme la connexion.

Le jeton est un fichier de 32 octets aléatoires en hexadécimal, créé en `0600` (`--token-file`, défaut `<dossier du profil>/daemon.token`). Un fichier lisible par le groupe ou les autres est refusé.

## 4. Appels

`call` est exécuté par le moteur dans une tâche indépendante : plusieurs appels peuvent être en vol sur une connexion, les réponses arrivent dans n'importe quel ordre et s'apparient par `id`. Une requête illisible reçoit une enveloppe d'échec, pas une fermeture. Une réponse plus grande que la taille maximale d'une trame devient une enveloppe d'échec au message fixe `réponse trop grande`. Si l'écriture vers le client échoue, la session se termine (les appels déjà lancés continuent). **Fermer la connexion n'annule jamais un appel** : un envoi de message ou une synchronisation lancés par un client se terminent même s'il disparaît.

Un `id` ne peut pas être réutilisé tant que son échange est en cours ; le faire ferme la connexion. `credit` et `cancel` sur un `id` inconnu (course avec un `end`) sont ignorés.

## 5. Flux

`open_events` et `open_chat` ouvrent un flux sous un `id`. Le client donne une **fenêtre de crédits** : chaque `event` ou `chat_delta` en consomme un, `credit` en ajoute.

- **Événements.** Sans `after`, seuls les événements futurs sont livrés (le journal passé n'est pas rejoué) ; avec `after: n`, le journal est rejoué à partir de `n` (exclu). Le démon garde une file de 256 événements par flux. Si elle reste pleine plus de 5 secondes (le client ne lit plus ou n'a plus de crédit), le flux se termine par `end` `lagged` : le client se réabonne avec `after` égal au `seq` du dernier événement reçu, sans perte ni doublon.
- **Chat.** Chaque ligne attend un crédit : sans crédit, la lecture du fournisseur est suspendue, pas abandonnée. `cancel`, ou la fermeture de la connexion, annule l'opération (le fournisseur voit sa connexion fermée, l'opération passe à `cancelled`) ; le flux se termine par `end` `cancelled`. L'annulation est constatée à la ligne suivante, comme pour l'ancien pont.
- Un flux terminé normalement envoie `end` `finished`. Sur un moteur jamais ouvert, un chat reçoit une seule ligne `error` au message fixe puis `end`.

## 6. Opérations d'hôte

`host_op` `shutdown` répond `result` (succès) puis arrête le démon (un seul arrêt à la fois, même demandé deux fois ou par un signal en même temps) : il cesse d'accepter, ferme les connexions (une seconde pour écrire ce qui est en file), laisse finir les appels en vol (5 secondes au plus), ferme le moteur et libère le verrou du profil. Toute connexion authentifiée peut l'envoyer. `set_poll_secs` et `retry_unlock` viendront avec les phases suivantes.

## 7. Le démon

`mira-daemon [options]` : `--profile`, `--home`, `--poll-secs`, `--secrets-key-file` (32 octets, `0600`, remplace le trousseau du système par un fichier chiffré), `--socket`, `--tcp-port`, `--token-file`. Pas de fichier de configuration avant la phase 4.

Il écoute d'abord (les clients voient `starting`), ouvre ensuite le moteur, écrit `tcp_port=<n>` (si TCP) puis `mira-daemon prêt` sur stdout. SIGTERM et SIGINT déclenchent le même arrêt que `shutdown`.

| Code de sortie | Sens |
|---|---|
| 0 | arrêt normal |
| 1 | échec (socket ou port inutilisable, ouverture du moteur impossible) |
| 2 | configuration invalide (chemins, jeton, fichier de clé) |
| 3 | un autre démon tient déjà la socket ou le profil (D-27) |

Une socket laissée par un démon tué est remplacée au redémarrage ; une socket sur laquelle quelqu'un écoute n'est jamais volée.

## 8. Sécurité

Aucune trame émise par le démon ne contient un secret (mots de passe, clés d'IA, jetons OAuth, jeton local) ; seul `has_key` dit qu'une clé existe. Les secrets que le contrat accepte déjà **en entrée** (mot de passe de `AddAccount`, clé de `SetAiApiKey`) restent permis dans les `call` entrants : ces transports sont locaux et authentifiés. Le test `secrets_in_frames` capture les octets émis par le démon pendant un scénario complet.

## 9. Où sont les tests

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
