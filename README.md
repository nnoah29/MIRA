# MIRA — moteur (engine) Rust

MIRA (*Mail Interface & Retrieval Application*) est un client mail Linux natif, multi-comptes et **offline-first** : le moteur tient un miroir local (SQLite + index plein texte FTS5) de chaque boîte IMAP, toutes les lectures se font sur ce miroir, et toutes les écritures sont d'abord enregistrées dans une file d'actions locale avant d'être rejouées vers le serveur (IMAP pour l'état des messages, SMTP pour l'envoi).

Ce dépôt contient le **moteur** seul, utilisable sans interface : une bibliothèque asynchrone (la façade `Engine`, contrat d'API versionné) et un client en ligne de commande, `mira`. Une interface graphique viendra plus tard au-dessus du même contrat.

- Architecture de référence (source de vérité, lecture seule) : `docs/MIRA-architecture-reference.md`.
- Ce qui a réellement été implémenté : `docs/ARCHITECTURE-IMPLEMENTEE.md`.
- Écarts proposés à la référence : `docs/PROPOSITIONS-REFERENCE.md`.
- Lire le code en venant du C++ : `docs/GUIDE-RUST-POUR-CPP.md`.

## Prérequis

- `rustup` : la toolchain **Rust 1.99.0** (édition 2024) est installée automatiquement grâce à `engine/rust-toolchain.toml` (avec `rustfmt` et `clippy`).
- `python3` (les garde-fous `tools/check-norms` et `tools/check-architecture` sont des scripts Python).
- Aucune bibliothèque système : SQLite est compilé avec le moteur et le trousseau passe par D-Bus en Rust pur (`keyring`). Pour utiliser le CLI avec de vrais comptes il faut en revanche un service *Secret Service* (par exemple GNOME Keyring ou KWallet) ; les tests, eux, n'y touchent jamais.

## Construire, tester, vérifier

```bash
cd engine
cargo build -p mira-cli            # produit engine/target/debug/mira
cargo test --workspace             # tous les tests
cargo run -q -p mira-cli -- --help # lancer le CLI sans l'installer
```

Depuis la racine, `./ci.sh` (ou `make check`, point d'entrée prévu par `docs/GIT_WORKFLOW.md`) exécute toute l'intégration continue locale ; tout doit être vert avant chaque commit. Il contrôle, dans l'ordre :

1. les auto-tests des garde-fous (`tools/test-check-norms`, `tools/test-check-architecture`) ;
2. `tools/check-norms` : normes de codage (≤ 30 lignes par fonction, ≤ 6 fonctions par fichier, aucun commentaire dans un corps de fonction, une ligne `///` avant chaque fonction, pas de fichier `utils`/`helpers`/`common`/`misc`) ;
3. `tools/check-architecture` : frontière cœur / providers (le cœur ne dépend d'aucun provider et ne nomme aucun fournisseur, les providers ne dépendent que de `mira-vocabulary`, chaque bibliothèque externe reste dans son module d'encapsulation, pas de chargement dynamique) ;
4. `tools/check-forbidden-patterns` : pas d'`unwrap`/`expect`/`panic!`/`todo!`/`dbg!`/`unsafe` hors tests, pas de `TODO` non référencé, pas de marqueur de conflit git, pas de `libloading` ;
5. `cargo fmt --all --check` ;
6. `cargo clippy --workspace --all-targets --all-features -- -D warnings` ;
7. les tests du workspace ;
8. `cargo deny check bans licenses sources` si `cargo-deny` est installé.

Deux précisions sur cet environnement :

- `ci.sh` utilise `cargo nextest` s'il est présent et **sinon `cargo test`** ; c'est `cargo test` qui sert ici, car `cargo-nextest` n'a pas pu être installé. Le journal des tests est écrit dans `target/rapports/tests.log`.
- `cargo deny` est lancé **sans les avis de sécurité** (`advisories`) : les avis de sécurité sont exclus car ils demandent un accès réseau (`ci.sh` le dit : « avis de sécurité hors ligne exclus ») ; seuls les interdits (`libloading`, `openssl`…), les licences et les sources de dépendances sont vérifiés (voir `engine/deny.toml`).

Tous les tests réseau tournent contre les faux serveurs du crate `mira-testkit` (IMAP, SMTP, OAuth2) sur `127.0.0.1`. **Aucun compte réel n'est utilisé dans ce dépôt**, et aucun test automatique ne touche au vrai trousseau.

## Organisation du dépôt

```
CLAUDE.md  Makefile  ci.sh
tools/      garde-fous : check-norms, check-architecture, check-forbidden-patterns (+ leurs auto-tests)
docs/       référence, décisions (DECISIONS-AGENT.md), specs/ par jalon, ETAT/ (journal, plan, invariants, tests)
engine/     workspace Cargo (toolchain 1.99.0, édition 2024)
  crates/mira-vocabulary   types du domaine, erreurs et événements du contrat, interface Provider (serde + thiserror seulement)
  crates/mira-core         le moteur : stockage SQLite/FTS5, profil et verrou, client IMAP, analyse MIME, SMTP,
                           OAuth2 PKCE, HTML sûr, synchro, file d'actions et rejeu, événements, façade api::Engine
  crates/mira-provider-*   gmail, outlook, yahoo, generic : descriptions pures (aucune entrée-sortie)
  crates/mira-assembly     composition root : enregistre les providers, fabrique la configuration de production
  crates/mira-cli          binaire `mira`
  crates/mira-testkit      faux serveurs IMAP / SMTP / OAuth pour les tests
```

Règle de frontière : le cœur ne nomme aucun fournisseur, les providers sont purs, seul `mira-assembly` connaît tout le monde (vérifié par `tools/check-architecture`).

## Le CLI `mira`

`mira` est un client de **courte durée** : chaque commande ouvre le moteur sur un profil, exécute une opération du contrat d'API, affiche le résultat, referme le moteur. Il y a une sous-commande par opération du contrat : `mira <domaine> <verbe> [options]`. `mira --help` et `mira <domaine> --help` donnent la liste à jour.

### Options globales et environnement

| Élément | Effet |
|---|---|
| `--json` | Sortie sous forme d'enveloppe JSON versionnée (voir plus bas), succès comme erreurs, sur stdout. |
| `--profile <nom>` | Profil (dossier de données) à utiliser ; défaut `default`. Un nom contenant `/`, `\` ou égal à `.`/`..` est refusé (code 10). |
| `-v`, `-vv` | Journalisation sur stderr : `-v` niveau info, `-vv` debug ; sans option, avertissements seulement. |
| `MIRA_LOG` | Directive de filtre `tracing` (par exemple `MIRA_LOG=mira_core=debug`) ; **prioritaire** sur `-v`. |
| `MIRA_HOME` | Racine des données : le profil vit dans `$MIRA_HOME/profiles/<profil>`. Sinon `$XDG_DATA_HOME/mira/profiles/<profil>`, sinon `~/.local/share/mira/profiles/<profil>`. |

Un profil contient la base `mira.sqlite3`, le cache de pièces jointes et un dossier temporaire (droits `0700`). Un profil est **exclusif** : un second `mira` sur le même profil, tant que le premier tourne, reçoit une erreur « profil déjà utilisé » (utiliser `--profile` pour un second profil). Les secrets ne sont jamais dans le profil : ils vont dans le trousseau du système.

### Secrets, clés d'idempotence, attentes

- **Secrets** : uniquement par l'entrée standard, avec `--password-stdin` (première ligne, 4 Ko au plus). Aucune option ne prend un mot de passe en argument, car il serait visible dans `ps` (`--password=x` est refusé, code 10). Le secret est rangé dans le trousseau, jamais renvoyé, ni journalisé, ni mis dans un événement ou une erreur.
- **`--key <clé>`** : clé d'idempotence des écritures (`account add`, `action apply`, `mailbox create|empty|…`, `draft send`). Rejouer la même clé ne crée qu'un compte ou qu'une action. Sans `--key`, une clé est fabriquée : pour pouvoir relancer sans risque une commande interrompue, passez toujours la vôtre.
- **Écritures = actions en file** : une écriture rend tout de suite l'identifiant d'une action, même hors ligne. **`--wait`** (avec **`--timeout <s>`**, 60 s par défaut) attend que l'action soit rejouée vers le serveur. Disponible sur `action apply`, `mailbox create|rename|delete|empty`, `draft send|resolve-send` et `queue retry`. Sans `--wait` la commande rend la main aussitôt : l'action reste en file, et le rejeu de fond ne vit que le temps du processus.
- **Opérations longues** (`message fetch`, `message save-attachment`, `search server`, `sync start|extend-window|resync`) : toujours attendues, avec `--timeout <s>` (120 s par défaut).
- **Instants** (`--since`, `--until`, `draft send --at`) : millisecondes Unix, `AAAA-MM-JJ` ou `AAAA-MM-JJTHH:MM[:SS]`, toujours en UTC.

### Codes de sortie

| Code | Catégorie d'erreur du contrat |
|---|---|
| 0 | succès (l'aide et la version sortent aussi en 0) |
| 10 | entrée invalide (dont les erreurs d'usage de la ligne de commande) |
| 11 | introuvable |
| 12 | état incompatible avec l'opération |
| 13 | réseau (dont un délai d'attente dépassé : l'action ou l'opération continue, rien n'est perdu) |
| 14 | TLS |
| 15 | authentification refusée (le compte demande une réauthentification) |
| 16 | erreur du serveur distant (dont une action rejouée en échec) |
| 17 | stockage local |
| 18 | trousseau de secrets |
| 19 | annulé |
| 84 | erreur interne |

### Enveloppe JSON

Avec `--json`, la sortie est une ligne JSON qui reflète le contrat d'API, avec un code de sortie identique :

```text
$ mira --json engine info
{"contract_version":{"major":1,"minor":0},"data":{"data":{"engine_capabilities":["imap","smtp","oauth2","app_password","fts5","safe_html"],"version":{"major":1,"minor":0}},"type":"contract_info"},"ok":true}

$ mira --json mailbox list inconnu ; echo $?
{"contract_version":{"major":1,"minor":0},"error":{"category":"not_found","code":"not_found.account","context":{"account":"inconnu",...},"message":"compte inconnu","retryable":false,"suggested_action":null},"ok":false}
11
```

- Succès : `{"contract_version": {...}, "ok": true, "data": ...}` ; pour les réponses du moteur, `data` est lui-même de la forme `{"type": "<variante>", "data": <charge utile>}`.
- Échec : `{"contract_version": {...}, "ok": false, "error": {category, code, message, retryable, suggested_action, context}}`. Le `code` est stable (`<catégorie>.<sujet>`), `suggested_action` propose par exemple `reauthenticate`.
- Sans `--json`, le résultat est un texte lisible sur stdout et les erreurs sont écrites sur stderr (`erreur : <message> (<code>)`).
- Les identifiants (compte, boîte, message, action…) sont des chaînes **opaques** : ne les fabriquez pas, relisez-les dans les sorties.

### Comportement d'un CLI de courte durée

- Les lectures sont locales (miroir) et marchent hors ligne. Le moteur est ouvert, la commande est exécutée, le moteur est refermé.
- **La synchronisation de fond ne tourne que pendant `sync start`, `sync extend-window`, `sync resync`, et `account add|add-oauth --sync-wait`.** Toute autre commande n'ouvre aucune connexion de synchro ; pour garder la boîte à jour il faut relancer `mira sync start` (par exemple depuis `cron` ou un timer systemd). Il n'y a pas de commande de veille permanente (IDLE) dans le CLI.
- Chaque ouverture et fermeture du moteur émet des événements `engine_started` / `engine_stopping`, visibles dans `mira events`.
- OAuth2 : `mira account add-oauth` fait tout dans un seul processus (voir ci-dessous), car la table des flux OAuth vit en mémoire dans le moteur.

### Commandes par domaine

Les identifiants (`<compte>`, `<boite>`, `<msg>`…) sont ceux rendus par les commandes de liste.

**engine** — le moteur lui-même

| Commande | Exemple |
|---|---|
| `engine info` — version du contrat et capacités | `mira engine info` |
| `engine status` — profil, dossier de données, comptes, actions en attente | `mira engine status` |

**provider** — catalogue (local, sans ouvrir de profil)

| Commande | Exemple |
|---|---|
| `provider list` — providers connus, repli en dernier | `mira provider list` |
| `provider detect <adresse>` — provider et serveurs d'une adresse | `mira provider detect alice@gmail.com` |

**account** — comptes

| Commande | Exemple et options principales |
|---|---|
| `account prepare` — propose provider et serveurs | `mira account prepare --address alice@yahoo.fr` ; `--provider` |
| `account test` — teste la connexion sans rien enregistrer | `mira account test --address alice@example.org --password-stdin` ; mêmes options que `add` |
| `account add` — ajoute un compte (test de connexion, secret au trousseau, identité par défaut, première synchro lancée) | `mira account add --address alice@yahoo.fr --auth app-password --password-stdin --key alice-1 --sync-wait` ; `--display-name`, `--provider`, `--auth password\|app-password\|oauth`, `--flow` (avec `--auth oauth`), `--imap-host/-port/-security`, `--smtp-host/-port/-security` (avec `--imap-host` et `--smtp-host` : serveurs saisis à la main, `-security` = `tls`, `starttls` ou `plaintext`), `--sync-wait`, `--timeout` |
| `account add-oauth` — autorise MIRA par OAuth2 puis ajoute le compte | `mira account add-oauth --address alice@gmail.com --sync-wait` ; mêmes options serveurs/`--key`/`--timeout` (300 s) |
| `account update <compte>` | `mira account update <compte> --display-name "Alice" --window-days 90 --mailbox <boite>` ; `--mailbox` (identifiant de boîte, répétable) remplace la liste des boîtes synchronisées ; options serveurs, sans effet sur ce qui n'est pas donné |
| `account pause <compte>` / `account resume <compte>` | `mira account pause <compte>` |
| `account delete <compte>` — supprime le compte, ses données locales et ses secrets ; rien n'est envoyé au serveur | `mira account delete <compte>` |
| `account list` | `mira account list` |

**identity** — identités d'envoi

| Commande | Exemple |
|---|---|
| `identity list <compte>` | `mira identity list <compte>` |
| `identity add <compte>` | `mira identity add <compte> --address alias@example.org --display-name "Alias" --signature "Alice" --default` |
| `identity update <identité>` | `mira identity update <identité> --display-name "Alice D." --signature "Alice"` |
| `identity set-default <identité>` | `mira identity set-default <identité>` |

**auth** — authentification et secrets

| Commande | Exemple |
|---|---|
| `auth oauth-start <adresse>` — démarre un flux, affiche l'URL d'autorisation | `mira auth oauth-start alice@gmail.com` ; `--provider` |
| `auth oauth-complete <flux>` / `auth oauth-cancel <flux>` | `mira auth oauth-complete <flux>` |
| `auth app-password <compte>` — remplace le mot de passe d'application | `printf '%s\n' "$MOT_DE_PASSE" \| mira auth app-password <compte> --password-stdin` |
| `auth reauthenticate <compte>` — ouvre un nouveau flux OAuth2 pour un compte OAuth2 (autre méthode : code 10, utiliser `auth app-password`) | `mira auth reauthenticate <compte>` |

**operation** — opérations longues

| Commande | Exemple |
|---|---|
| `operation get <opération>` — nature et avancement | `mira operation get <opération>` |
| `operation cancel <opération>` — demande l'annulation | `mira operation cancel <opération>` |

**mailbox** — boîtes d'un compte

| Commande | Exemple et options principales |
|---|---|
| `mailbox list <compte>` — boîtes et compteurs | `mira mailbox list <compte>` |
| `mailbox create <compte> <nom>` (action en file) | `mira mailbox create <compte> Factures --parent <boite> --key mb-1 --wait` |
| `mailbox rename <boîte> <nom>` | `mira mailbox rename <boite> Archives-2026 --wait` |
| `mailbox delete <boîte>` / `mailbox empty <boîte>` | `mira mailbox empty <boite> --key vider-1 --wait` |
| `mailbox set-synced <compte> [boîtes…]` — boîtes à synchroniser (aucune : plus aucune) | `mira mailbox set-synced <compte> <boite1> <boite2>` |

`create`, `rename`, `delete`, `empty` acceptent `--key`, `--wait`, `--timeout`.

**message** — lecture

| Commande | Exemple et options principales |
|---|---|
| `message list` — liste locale paginée | `mira message list --mailbox <boite> --unread --limit 20` ; `--mailbox` ou `--view unified\|unread\|flagged` (exclusifs), `--unread`, `--flagged`, `--with-attachments`, `--from`, `--since`, `--until`, `--limit` (1–500, défaut 50), `--cursor` (page suivante) |
| `message get <msg>` — en-têtes et état | `mira message get <msg>` |
| `message body <msg>` — corps déjà téléchargé | `mira message body <msg>` |
| `message fetch <msg>` — télécharge le corps (réseau) | `mira message fetch <msg> --timeout 60` |
| `message raw <msg>` — source brute déjà téléchargée | `mira message raw <msg>` |
| `message attachments <msg>` | `mira message attachments <msg>` |
| `message save-attachment <pièce> <fichier>` — écriture atomique | `mira message save-attachment <pièce> ./facture.pdf` |
| `message safe-html <msg>` — HTML assaini, contenu distant bloqué | `mira message safe-html <msg>` |
| `message allow-remote` — autorise le contenu distant | `mira message allow-remote --sender news@example.org` ; ou `--message <msg>` |

**thread** — fils

| Commande | Exemple |
|---|---|
| `thread list <boîte>` — du plus récent au plus ancien | `mira thread list <boite> --limit 20 --cursor <curseur>` |
| `thread get <fil>` | `mira thread get <fil>` |

**search** — recherche

| Commande | Exemple et options principales |
|---|---|
| `search local <mots…>` — miroir local, hors ligne | `mira search local facture mars --from compta --since 2026-01-01` ; `--mailbox`, `--to`, `--unread`, `--flagged`, `--with-attachments`, `--until`, `--limit`, `--cursor` |
| `search server <compte> <mots…>` — recherche sur le serveur, attend le résultat | `mira search server <compte> facture --timeout 60` |
| `search server-result <opération>` | relit un résultat de recherche serveur du **même** processus : d'un autre processus il ne trouve rien (code 11) |

**maintenance** — profil

| Commande | Exemple |
|---|---|
| `maintenance stats` — volumes du miroir | `mira maintenance stats` |
| `maintenance purge-attachments` — vide le cache de pièces jointes (le serveur les garde) | `mira maintenance purge-attachments` |
| `maintenance rebuild-index` — reconstruit l'index plein texte | `mira maintenance rebuild-index` |
| `maintenance check` — intégrité de la base | `mira maintenance check` |
| `maintenance export-diagnostic <fichier>` — sans adresse ni secret | `mira maintenance export-diagnostic ./diag.json` |
| `maintenance export-profile <fichier>` — configuration et intentions, sans secret ni miroir | `mira maintenance export-profile ./profil.json` |

**action** — actions sur les messages

| Commande | Exemple et options principales |
|---|---|
| `action apply <nature>` — en lot, sur des messages ou sur le résultat d'une requête | `mira action apply mark-read --message <msg> --key lu-1 --wait` ; natures : `mark-read`, `mark-unread`, `flag`, `unflag`, `move`, `add-to-mailbox`, `remove-from-mailbox`, `archive`, `trash`, `delete-permanently`, `spam`, `not-spam` ; `--message` (répétable) **ou** `--mailbox`/`--view` avec les filtres de `message list` ; `--to <boîte>` pour `move`/`add-to-mailbox`/`remove-from-mailbox` ; `--key`, `--wait`, `--timeout` |
| `action undo <action>` — annule une action encore en attente (délai de 5 s) | `mira action undo <action>` |

Exemple de sélection par requête : `mira action apply archive --mailbox <boite> --unread --from newsletter --key arch-1`.

**draft** — rédaction et envoi

| Commande | Exemple et options principales |
|---|---|
| `draft create <compte>` — neuf, réponse ou transfert | `mira draft create <compte> --reply <msg>` ; `--reply`, `--reply-all`, `--forward` (exclusifs), `--identity` |
| `draft update <brouillon>` | `mira draft update <brouillon> --to bob@example.org --subject "Bonjour" --body "Texte"` ; `--cc`, `--bcc`, `--html`, `--body-file`, `--identity` ; destinataires `adresse` ou `Nom <adresse>`, répétables ; valeur vide = efface |
| `draft attach <brouillon> <fichier>` / `draft detach <brouillon> <position>` | `mira draft attach <brouillon> ./devis.pdf` |
| `draft get <brouillon>` / `draft list <compte>` / `draft delete <brouillon>` | `mira draft list <compte>` |
| `draft send <brouillon>` — programme l'envoi, annulable 10 s | `mira draft send <brouillon> --key envoi-1 --wait` ; `--at <instant>` (envoi différé) |
| `draft cancel-send <action>` — annule un envoi encore en attente | `mira draft cancel-send <action>` |
| `draft outbox <compte>` — boîte d'envoi | `mira draft outbox <compte>` |
| `draft resolve-send <action>` — tranche un envoi **incertain** | `mira draft resolve-send <action> --resend --wait` ; sans `--resend` l'envoi est tenu pour parti |

**queue** — file d'actions locale

| Commande | Exemple et options principales |
|---|---|
| `queue list <compte>` | `mira queue list <compte> --state pending --state failed` ; états : `pending`, `in-progress`, `succeeded`, `failed`, `cancelled`, `uncertain` |
| `queue retry <action>` — relance une action échouée | `mira queue retry <action> --wait` |
| `queue cancel <action>` — annule une action encore en attente | `mira queue cancel <action>` |

**sync** — synchronisation

| Commande | Exemple et options principales |
|---|---|
| `sync start` — synchronise tout, un compte ou une boîte, et attend la fin | `mira sync start --account <compte>` ; `--account` ou `--mailbox` (exclusifs), `--timeout` ; `--wait` est accepté mais sans effet (le CLI attend toujours) |
| `sync status` — état de chaque compte | `mira sync status` |
| `sync pause --account <compte>` | `mira sync pause --account <compte>` |
| `sync extend-window --mailbox <boîte> --days <n>` — remonte la plage de synchro de `n` jours **en plus** | `mira sync extend-window --mailbox <boite> --days 90` |
| `sync resync --mailbox <boîte>` — relit entièrement la boîte | `mira sync resync --mailbox <boite>` |

**events** — journal d'événements numérotés

| Commande | Exemple |
|---|---|
| `events` — événements du moteur, sans secret ni contenu | `mira events --after 120 --limit 50 --account <compte>` (`--account` répétable) |

### Comptes OAuth2 (Gmail, Outlook)

`mira account add-oauth --address alice@gmail.com` déroule un seul processus : démarrage du flux, affichage de l'URL d'autorisation sur **stderr**, attente de la redirection sur une boucle locale (`127.0.0.1`, PKCE), échange du code contre les jetons (rangés au trousseau), puis ajout du compte. Après le délai (`--timeout`, 300 s), le flux est annulé et la commande sort en 13. Les jetons sont rafraîchis silencieusement ; en cas d'échec le compte passe en « authentification requise ». Pour un compte à mot de passe d'application, on le remplace avec `auth app-password`.

> Limite : le binaire de production ne contient **aucun identifiant d'application OAuth** (`OAuthClientConfig::default()` dans `mira-assembly`). Tant qu'un identifiant n'est pas fourni par la configuration, `account add-oauth` s'arrête en code 10 (« identifiant d'application OAuth absent de la configuration »). Les comptes à mot de passe ou mot de passe d'application ne sont pas concernés.

### Session d'exemple

Ajout d'un compte à mot de passe d'application, synchro, lecture, action, envoi. Les identifiants entre chevrons sont opaques : relisez-les dans les sorties (`--json` pour les extraire par script).

> Cette session est celle que couvrent les tests du CLI (`engine/crates/mira-cli/tests/scenario_s02_*`, `scenario_s04_*`, `scenario_s07_*`, `scenario_s09_*`…), qui l'exécutent **en processus contre les faux serveurs du testkit** avec un faux trousseau. Ce dépôt ne contient aucun compte réel et n'a jamais été exercé contre un vrai service ; les identifiants et adresses ci-dessous sont fictifs.

```bash
# 1. Quel provider, quels serveurs ?
mira provider detect alice@yahoo.fr
mira account prepare --address alice@yahoo.fr

# 2. Ajouter le compte : le mot de passe d'application passe par stdin, jamais en argument
read -rsp "Mot de passe d'application : " MIRA_PW; echo
printf '%s\n' "$MIRA_PW" | mira account add --address alice@yahoo.fr \
    --auth app-password --password-stdin --key alice-1 --sync-wait
unset MIRA_PW
mira account list                       # note <compte>

# 3. Synchroniser et lire (lectures locales, hors ligne possible)
mira sync start --account <compte>
mira mailbox list <compte>              # note <boite> (la réception)
mira message list --mailbox <boite> --unread --limit 20
mira message fetch <msg>                # télécharge le corps si besoin (réseau)
mira message body <msg>

# 4. Agir : l'action est mise en file, puis rejouée vers le serveur
mira action apply mark-read --message <msg> --key lu-1 --wait
mira action apply archive --message <msg> --key arch-1     # sans --wait : reste en file
mira queue list <compte> --state pending

# 5. Répondre et envoyer
mira draft create <compte> --reply <msg>                   # note <brouillon>
mira draft update <brouillon> --body "Merci, bien reçu."
mira draft send <brouillon> --key envoi-1 --wait           # part après 10 s (annulable), copie dans Envoyés
mira draft outbox <compte>

# 6. Suivre
mira events --after 0 --limit 20
```

Si le réseau tombe pendant l'étape 4, les actions restent en file ; `mira queue list <compte>` les montre et une commande ultérieure avec `--wait` ou `queue retry --wait` les rejoue. Un envoi dont l'issue est inconnue (coupure après l'envoi des données) n'est **jamais renvoyé automatiquement** : il apparaît `uncertain` et se tranche avec `draft resolve-send`.

### Limites connues

- Aucun test contre un vrai serveur ni un vrai trousseau (règle de la session de développement) ; le comportement face aux particularités réelles des fournisseurs reste à valider.
- Pas de veille permanente : pas d'IDLE tenu par le CLI, pas de commande `watch` ; la mise à jour passe par `mira sync start`.
- OAuth2 : pas d'identifiant d'application dans le binaire (voir plus haut) ; `auth oauth-start` puis `auth oauth-complete` dans deux processus différents est impossible (la table des flux est en mémoire du moteur) : utiliser `account add-oauth`. Pour la même raison `auth reauthenticate` (qui ouvre un flux) ne peut pas être terminée par un second processus.
- `search server-result` ne retrouve que le résultat d'une recherche faite dans le même processus.
- `--wait` sur un compte qui demande une réauthentification sort aussitôt en code 15 (action suggérée `reauthenticate`) ; l'action reste en file.
- Les noms de boîtes IMAP en UTF-7 modifié (`&AOk-`) ne sont pas décodés ; `html::sanitize_html` est quadratique en profondeur d'imbrication (un message hostile très profond peut ralentir `safe-html`). Ces deux points sont consignés dans `docs/DECISIONS-AGENT.md` (AG-34), qui donne l'état exact.
- L'effacement d'un compte est durci (`secure_delete`, checkpoint du journal) mais ne purge pas rétroactivement les traces d'une base créée avant ce correctif (AG-33).

## Documentation

| Fichier | Contenu |
|---|---|
| `docs/MIRA-architecture-reference.md` | Architecture de référence (lecture seule) |
| `docs/ARCHITECTURE-IMPLEMENTEE.md` | Ce qui est réellement implémenté, module par module |
| `docs/GUIDE-RUST-POUR-CPP.md` | Lire le code Rust quand on vient du C++ |
| `docs/DECISIONS-AGENT.md` | Décisions d'implémentation (contexte / choix / raison / retour arrière) |
| `docs/PROPOSITIONS-REFERENCE.md` | Corrections proposées à la référence |
| `docs/PASSATION.md`, `docs/RAPPORT-FINAL.md` | Passation et bilan de la session de développement |
| `docs/specs/` | Spécifications par jalon (dont `J8-J9-facade-cli.md` pour le CLI) |
| `docs/ETAT/` | Journal, plan, invariants et scénarios, résultats de tests, décisions de départ |
| `docs/GIT_WORKFLOW.md` | Conventions Git (lecture seule) |
