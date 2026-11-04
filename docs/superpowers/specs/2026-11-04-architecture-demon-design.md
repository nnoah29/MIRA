# Architecture démon + client : le moteur hors de l'interface

> **Statut :** spec à relire. Elle remplace la réflexion `docs/dev/REFLEXION-ARCHITECTURE-DEMON.md` (qui reste l'historique de la discussion). Le spike proposé dans la réflexion est abandonné : on passe directement à l'implémentation par phases. Les choix marqués **[à confirmer]** sont des valeurs par défaut prises faute de réponse.

## 1. Objectif et périmètre

Faire tourner le moteur dans un **processus qui survit à l'interface**, pour que la synchronisation et les notifications continuent app fermée, et préparer le **moteur distant** (VPS, app sur un autre poste), objectif de la version 4.

| Plateforme | Modèle | Transport par défaut |
|---|---|---|
| Linux | Démon séparé | Socket Unix `0600` dans `XDG_RUNTIME_DIR` |
| macOS | Démon séparé (agent `launchd`) | Socket Unix |
| Windows | Démon séparé | TCP `127.0.0.1` + jeton (pas de named pipe, voir §3.3) |
| Android, iOS | Moteur embarqué (pont FFI actuel) | Aucun |
| Bureau, version 4 | Moteur distant | TCP + TLS, appairage (§8) |

Hors périmètre : la cible web ; le packaging Flatpak/Snap (noté §10) ; l'IA ou la logique métier du moteur, qui ne changent pas.

**Invariant directeur :** le contrat JSON (`CONTRACT_VERSION`, chap. 12 de la référence) reste strictement identique. Seul le transport change. Le moteur reste une bibliothèque embarquable ; le démon est un hôte de plus.

## 2. Découpage en crates

```
engine/crates/
  mira-host       logique d'hôte commune au pont FFI et au démon
  mira-protocol   trames du protocole (types + codec, sans I/O)
  mira-daemon     binaire : écoute les transports, applique le protocole, héberge le moteur
bridge/           reste hors workspace ; ne garde que les enveloppes FRB et dépend de mira-host
```

- **`mira-host`** reprend ce que le pont contient aujourd'hui : `runtime`, `envelope`, `chat_feed`, `events_feed`, `original_html`, `file_secrets`, `platform`, `polling`. Le moteur n'y est plus un `static` global (`session.rs`) mais une valeur possédée par un objet `Host` ; le pont FFI en garde une instance globale, le démon une instance par processus. Les tests de `bridge/tests` suivent le code (changement de chemins d'import seulement).
- **`mira-protocol`** est pur : types de trames, encodage, décodage, validation de taille. Il sert de source de vérité écrite du format, que le client Dart réimplémente.
- **`mira-daemon`** est le seul crate qui connaît les sockets, TLS et le cycle de vie du processus.
- Les règles du dépôt s'appliquent (frontière cœur/providers, pas d'`unsafe`, pas d'`unwrap` hors tests, `tools/check-norms`). `mira-host` et `mira-protocol` ne nomment aucun provider.

## 3. Transport

### 3.1 Abstraction côté Rust

Un transport ne fournit qu'un **flux d'octets bidirectionnel fiable** :

- `Listener::accept() -> Connection` avec `Connection: AsyncRead + AsyncWrite + Send`, plus une description de confiance (`Trust::LocalUser(uid)` ou `Trust::Remote`) que le protocole consulte pour sa politique d'accès (§6, §8).
- Implémentations : `UnixListener` (socket `0600`, vérification de l'uid du pair avec `peer_cred`, sans `unsafe`), `LoopbackTcpListener` (jeton), plus tard `TlsListener` (v4) et `NamedPipeListener` (si le besoin se confirme).

### 3.2 Abstraction côté Dart

`EngineClient` reste l'interface haute. Deux implémentations : `RustEngineClient` (FFI, inchangé) et `RemoteEngineClient` (nouveau). Sous `RemoteEngineClient`, une interface basse `ByteChannel` (flux d'octets entrant, sink sortant, fermeture) a une implémentation par transport : `UnixSocketChannel` et `TcpChannel` via `dart:io`, `TlsChannel` en v4. Le protocole est écrit une fois, au-dessus de `ByteChannel`.

### 3.3 Windows

`dart:io` n'a pas de named pipe natif ; passer par `dart:ffi` ajouterait du code natif à maintenir. Version 1 : TCP loopback + jeton, comme transport unique sur Windows. Le named pipe reste une option ultérieure sans effet sur le protocole.

## 4. Protocole

Trames **JSON préfixées par leur longueur** (4 octets, grand-boutiste, taille maximale fixée côté serveur et rejetée avant lecture du corps). Chaque trame a un champ `type` et, si elle appartient à un échange, un `id` (u64 choisi par le client).

| Type | Sens | Rôle |
|---|---|---|
| `hello` | C→S | `protocol_version`, `contract_version`, `client_id`, `client_name`, `auth` |
| `hello_ok` | S→C | versions du démon, `host_state` (§5), ou `hello_err` avec raison |
| `call` / `result` | C→S / S→C | une requête du contrat ; réponse = l'enveloppe actuelle (`success` / `failure`) |
| `open_events` | C→S | `after` optionnel, comme `watch_events` |
| `open_chat` | C→S | `ChatStart`, comme `chat_stream` |
| `event`, `chat_delta` | S→C | un élément du flux `id` |
| `credit` | C→S | fenêtre de contre-pression du flux `id` (n trames autorisées) |
| `cancel` | C→S | annule l'échange `id` |
| `get_original_html` | C→S | réponse en `result` |
| `end` | S→C | fin d'un flux (raison : terminé, annulé, `lagged`, erreur) |
| `ping` / `pong` | les deux | détection de connexion morte |

Règles :

- **Concurrence :** plusieurs `call` en vol sur une même connexion, appariés par `id`.
- **Contre-pression :** par flux, par crédits. Un flux d'événements dont le client ne suit plus est fermé par `end{lagged}` ; le client se réabonne avec `after` = dernier numéro reçu (la reprise existe déjà). Un flux de chat bloque la génération, comme le canal borné actuel (32).
- **Annulation :** `cancel`, ou fermeture de la connexion, qui annule tous ses flux. La règle actuelle « fermer le flux Dart annule la génération » (invoquée par `CancelOperation`) devient « fermer l'échange ou la connexion ».
- **Une déconnexion n'annule jamais un envoi ni une synchronisation en cours :** ils appartiennent au moteur, pas au client (la reprise sur crash existante le garantit aussi).
- **Poignée de main :** refus si `protocol_version` majeure différente ou si le contrat est incompatible selon la règle de `CONTRACT_VERSION` (à relire dans `mira-vocabulary` au début de la phase 2, sans en inventer une autre).
- **Pas de secrets dans les trames :** les clés IA ne sortent jamais (`has_key` seulement) ; test dédié (§9).

## 5. Possession du moteur et état de l'hôte

- Le démon possède le moteur. Le client **s'attache** (`hello`) et ne l'ouvre plus. `close_engine` côté client devient un simple détachement ; l'arrêt du démon est une opération d'hôte réservée (`shutdown`).
- `home`, `profile`, `secrets_key`, `poll_secs` deviennent la configuration du démon (fichier de configuration dans le dossier MIRA + options de ligne de commande). **[à confirmer]** Le réglage de `poll_secs` par l'app passe par une opération d'hôte (`set_poll_secs`), persistée et prise en compte sans redémarrage. Deux clients : le dernier réglage gagne, l'état courant est annoncé dans `hello_ok`.
- Le verrou de profil (D-27) garantit un seul démon par profil. Un second lancement du démon sur le même profil échoue proprement, et l'app s'attache au premier.
- `host_state` annoncé à l'attache : `ready`, `starting`, `keyring_locked` (trousseau verrouillé, aucun écran pour demander le mot de passe : l'app l'affiche et envoie `retry_unlock`), `update_required`.

## 6. Authentification et sécurité

- Toute connexion s'authentifie dans `hello`, même locale. Socket Unix : permissions + uid du pair. TCP loopback : jeton aléatoire (≥ 256 bits) dans un fichier `0600` du dossier MIRA, comparé en temps constant, jamais journalisé.
- Le démon n'écoute que sur `127.0.0.1` tant que le transport distant n'est pas explicitement activé (v4).
- La politique d'accès dépend de `Trust` : un client `Remote` n'obtient jamais d'opération qui suppose le système de fichiers local (voir §8).
- Aucune donnée sensible dans les journaux, les erreurs d'authentification (message fixe), ni les trames.

## 7. Cycle de vie du démon (bureau)

1. Au lancement, l'app cherche un démon vivant ; sinon elle le démarre (unité `systemd --user` sous Linux, agent `launchd` sous macOS, tâche de session sous Windows) ou, à défaut, repasse en moteur embarqué. **[à confirmer]** Le repli embarqué reste disponible tant que le démon n'est pas livré sur toutes les plateformes bureau.
2. Notifications, icône de zone de notification (tray) et « quitter / désactiver » sont portés par le démon (phase 4) ; le clic sur une notification relance l'app.
3. **Mise à jour :** à l'attache, si les versions ne sont pas compatibles, l'app propose de redémarrer le démon (`shutdown` puis relance) ; si d'autres clients sont attachés, elle l'indique avant. Le contrat versionné évite le redémarrage tant que la version majeure est la même.

## 8. Moteur distant (version 4) : exigences à respecter dès maintenant

La spec du distant viendra plus tard, mais ces points sont acquis dès la phase 1 pour ne rien refermer :

- **Pas de chemin local dans le contrat.** Un inventaire des requêtes et réponses qui portent un chemin de fichier (pièces jointes, import/export) est une tâche de la phase 1. Chaque cas trouvé est noté pour devenir un transfert par le protocole (envoi/téléchargement par flux) avant la v4.
- **`client_id` stable** dans `hello`, pour le multi-appareils et la révocation.
- **Authentification de transport remplaçable :** `auth` est un champ typé (`local_token`, plus tard `device_key`), pas une chaîne libre.
- **TLS (rustls, déjà dans l'arbre) + appairage** : un code à usage unique affiché par le démon, qui enregistre la clé publique de l'appareil ; révocation côté démon ; pas de mot de passe réutilisable.
- **Reprise après coupure :** déjà couverte pour les événements (`after`) ; à vérifier pour les appels en vol (rejouables car idempotents ou déclarés non rejouables).
- **Positionnement :** le moteur distant tourne sur un serveur choisi par l'utilisateur, jamais sur un service fourni par MIRA. À formuler dans `PROPOSITIONS-REFERENCE.md` avec la clôture de D-30.

## 9. Tests

- **Un seul jeu de tests de contrat, exécuté sur les deux chemins :** moteur embarqué et moteur derrière un transport de boucle locale (socket Unix en Rust). Aucun test n'est affaibli ni dupliqué en version « allégée ».
- **Protocole :** appels concurrents appariés, flux d'événements avec reprise par `after`, `lagged` suivi d'un réabonnement, annulation du chat par `cancel` et par fermeture de connexion, déconnexion pendant un envoi (l'envoi aboutit), poignée de main avec versions incompatibles, jeton faux, trame trop grande, trame tronquée.
- **Sécurité :** aucune trame ne contient un secret (clés IA, mots de passe, jetons OAuth) ; test sur les trames capturées d'un scénario complet.
- **Dart :** `RemoteEngineClient` contre le vrai démon sur socket Unix dans un test d'intégration ; `ByteChannel` simulé pour les cas d'erreur. Selon la consigne du projet, on cible les tests concernés et on ne lance la suite complète qu'en fin de chantier.

## 10. Phases

| Phase | Contenu | Critère de fin |
|---|---|---|
| **1. Hôte commun** | Crate `mira-host` ; pont réduit à ses enveloppes FRB ; inventaire des chemins locaux du contrat ; clôture de D-30 écrite dans `PROPOSITIONS-REFERENCE.md` et `DECISIONS-AGENT.md` | Tests du pont inchangés (hors imports) et verts ; app identique |
| **2. Protocole et démon** | `mira-protocol`, `mira-daemon`, transports Unix et TCP loopback, authentification locale, jeu de contrat sur les deux chemins | Contrat vert via socket ; tests de protocole verts |
| **3. Client Dart** | `ByteChannel`, protocole en Dart, `RemoteEngineClient`, choix démon/embarqué au démarrage, états `keyring_locked`/`update_required` | L'app tourne sur le démon sous Linux |
| **4. Démon résident** | Unité `systemd --user`, notifications, tray, démarrage au login, mise à jour | Fermer l'app : la synchro et les notifications continuent |
| **5. Plateformes** | Windows (TCP + jeton), macOS (`launchd`) | App fonctionnelle sur chacune |
| **v4. Distant** | Spec propre : TLS, appairage, transfert de fichiers, multi-appareils | Hors de cette spec |

Chaque phase a son plan d'implémentation et se termine par un commit par fonctionnalité. Android et iOS ne bougent pas : ils gardent le pont FFI.

## 11. Points ouverts qui restent à trancher

- Trousseau verrouillé sans interface : comportement exact (réessai périodique ou seulement sur `retry_unlock`).
- Flatpak/Snap : le démon doit vivre dans le même bac à sable que l'app, démarrage en arrière-plan par portail, chemins de pièces jointes non communs ; à traiter à l'empaquetage, après la phase 4.
- Cible web (`bridge_library_web.dart`) : considérée abandonnée pour ce chantier **[à confirmer]**.
