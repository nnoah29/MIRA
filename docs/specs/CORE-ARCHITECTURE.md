# Architecture implémentée du cœur (`mira-core`) — plan de référence pour les sous-agents

Ce document fixe la carte des modules, les ports injectables et le schéma de stockage, pour que plusieurs agents écrivent des parties différentes sans se contredire. Il applique D-33/D-34 (sous-domaines du cœur = modules) et l'annexe H (encapsulation de chaque bibliothèque externe).

## 1. Carte des modules de `mira-core/src/`

| Module | Responsabilité | Bibliothèque encapsulée | Jalon |
|---|---|---|---|
| `ports/` | Horloge, générateur d'identifiants, trousseau : traits + implémentations réelles + fausses (publiques, pour les tests des autres crates) | `uuid`, `keyring` | J2 |
| `profile/` | Emplacements d'un profil (XDG, `MIRA_HOME`), verrou exclusif (D-27) | `std::fs` | J2 |
| `storage/` | Base SQLite par profil, migrations, un dépôt (repository) par entité, recherche FTS5 | `rusqlite`, `zstd` | J2 |
| `registry/` | Registre des providers, détection, repli (6.6) | — | J3 |
| `capabilities/` | Capacités effectives = annonce du serveur − exceptions du provider (D-08, 10.14) | — | J3 |
| `mime/` | Source brute → structure (en-têtes, texte, HTML, pièces jointes), tolérant | `mail-parser` | J5a |
| `html/` | Assainissement HTML, liste des ressources distantes bloquées | `ammonia` | J10 |
| `threads/` | Calcul des fils (D-15) | — | J6 |
| `imap/` | Client IMAP de haut niveau (annexe H) | `async-imap` (ou client maison) | J5a |
| `smtp/` | Client SMTP | `lettre` | J7 |
| `auth/` | OAuth2 PKCE + écouteur de boucle locale, rafraîchissement ; mot de passe d'application | `reqwest`, `sha2`, `base64`, `rand` | J8 |
| `sync/` | Stratégies (D-22), descente par étapes (D-20), identité (D-13), renumérotation (10.5), IDLE, superviseur par compte | — | J5b |
| `actions/` | File d'actions, application locale optimiste, rejeu, idempotence, annulation (D-25) | — | J7 |
| `send/` | Envoi : préparation, délai, SMTP, copie dans Envoyés, état incertain | `mail-builder` | J7 |
| `events/` | Bus d'événements : persistance numérotée + diffusion (`tokio::sync::broadcast`) | `tokio` | J2/J8 |
| `accounts/` | Cycle de vie des comptes et identités | — | J8 |
| `api/` | Contrat : `Request`/`Response`, `Engine` (façade), un handler par opération | — | J8 |

Règles :
- Le cœur ne nomme jamais un provider (le garde-fou A6 cherche gmail/outlook/yahoo/google/microsoft… dans tout `mira-core`, tests compris : les faux providers des tests s'appellent `fake`, `folders_fake`, `labels_fake`).
- Une bibliothèque externe n'est importée **que** dans son module (par exemple `rusqlite` seulement sous `storage/`). Les autres modules utilisent les types du moteur.
- Les erreurs d'une bibliothèque sont converties en `MiraError` (vocabulaire) à la frontière du module, sans secret dans le message.
- Tests unitaires du cœur : `src/<module>/tests/<thème>.rs` déclarés par `#[cfg(test)] mod tests;` dans le `mod.rs` du module (le `tests/mod.rs` déclare les fichiers de thème). Tests d'intégration : `engine/crates/mira-core/tests/<thème>.rs`.

## 2. Ports injectables (`ports/`)

```rust
pub trait Clock: Send + Sync + std::fmt::Debug { fn now(&self) -> Timestamp; }
pub struct SystemClock;                         // horloge réelle
pub struct ManualClock { .. }                   // ManualClock::new(Timestamp), set(Timestamp), advance_millis(i64)

pub trait IdGenerator: Send + Sync + std::fmt::Debug { fn next_id(&self) -> String; }
pub struct UuidV7Ids;                           // UUID v7 (décision de départ)
pub struct SequentialIds { .. }                 // "id-000001", "id-000002"… déterministe pour les tests

pub struct SecretValue(String);                 // Debug et Display masqués ("***") ; expose(&self) -> &str ; pas de Serialize
pub trait SecretStore: Send + Sync + std::fmt::Debug {
    fn put(&self, key: &SecretRef, value: &SecretValue) -> Result<(), MiraError>;
    fn get(&self, key: &SecretRef) -> Result<Option<SecretValue>, MiraError>;
    fn delete(&self, key: &SecretRef) -> Result<(), MiraError>;
}
pub struct KeyringSecretStore;                  // SecretService via `keyring` ; erreurs → catégorie Secrets (keyring_unavailable / keyring_locked), JAMAIS de repli en clair (7.7)
pub struct MemorySecretStore { .. }             // tests uniquement ; `unavailable()` simule un trousseau indisponible
```

## 3. Profil (`profile/`)

- `ProfilePaths { root, database, attachments_cache, temp }` : racine = `$MIRA_HOME/profiles/<profil>` si `MIRA_HOME` est défini, sinon `$XDG_DATA_HOME/mira/profiles/<profil>` (défaut `~/.local/share`). Profil par défaut : `default` (D-09, D-10). La racine est créée avec des droits `0o700`.
- `ProfileLock` : fichier `<root>/engine.lock` verrouillé de façon exclusive avec `std::fs::File::try_lock` (stable depuis Rust 1.89, pas d'`unsafe`). Déjà verrouillé → `MiraError` `incompatible_state.profile_locked`. Libéré au `Drop`.

## 4. Schéma SQLite (`storage/`)

Une base `mira.sqlite3` par profil. `PRAGMA journal_mode=WAL; foreign_keys=ON; synchronous=NORMAL`. Migrations versionnées dans `storage/migrations/` (tableau ordonné de scripts, version dans `PRAGMA user_version`), appliquées dans une transaction.

Famille **configuration** (8.1) :
- `accounts(id TEXT PK, address TEXT, display_name TEXT, provider_id TEXT, auth_kind TEXT, servers_json TEXT NULL, sync_json TEXT, state_json TEXT, created_at INTEGER, deleting INTEGER DEFAULT 0)`
- `identities(id TEXT PK, account_id TEXT REFERENCES accounts ON DELETE CASCADE, address TEXT, display_name TEXT, signature TEXT NULL, is_default INTEGER)`
- `remote_content_allowances(kind TEXT CHECK(kind IN ('message','sender')), value TEXT, PRIMARY KEY(kind, value))`

Famille **miroir** (jetable, 8.4) :
- `mailboxes(id TEXT PK, account_id TEXT REFERENCES accounts ON DELETE CASCADE, name TEXT, display_name TEXT, delimiter TEXT NULL, parent_id TEXT NULL, role TEXT, sync_enabled INTEGER, uid_validity INTEGER NULL, uid_next INTEGER NULL, highest_modseq INTEGER NULL, total INTEGER DEFAULT 0, unread INTEGER DEFAULT 0, UNIQUE(account_id, name))`
- `messages(id TEXT PK, account_id TEXT REFERENCES accounts ON DELETE CASCADE, thread_id TEXT NULL, from_json, to_json, cc_json, bcc_json, reply_to_json TEXT, subject TEXT, date INTEGER NULL, internal_date INTEGER NULL, message_id_header TEXT NULL, in_reply_to TEXT NULL, references_json TEXT, seen INTEGER, flagged INTEGER, answered INTEGER, forwarded INTEGER, size INTEGER, preview TEXT, has_attachments INTEGER, unsubscribe_json TEXT NULL, content_state TEXT, sort_date INTEGER)` + index `(account_id, sort_date DESC, id)`.
- `appearances(message_id TEXT REFERENCES messages ON DELETE CASCADE, mailbox_id TEXT REFERENCES mailboxes ON DELETE CASCADE, uid INTEGER, deleted INTEGER DEFAULT 0, needs_revalidation INTEGER DEFAULT 0, PRIMARY KEY(message_id, mailbox_id), UNIQUE(mailbox_id, uid))` + index `(mailbox_id, message_id)`.
- `server_ids(account_id TEXT, kind TEXT CHECK(kind IN ('stable','uid')), value TEXT, message_id TEXT REFERENCES messages ON DELETE CASCADE, PRIMARY KEY(account_id, kind, value))` — table de correspondance D-13 ; pour `uid`, `value = "<mailbox_id>/<uid_validity>/<uid>"`.
- `raw_sources(message_id TEXT PK REFERENCES messages ON DELETE CASCADE, compressed BLOB, raw_size INTEGER)` — zstd (D-16).
- `parsed_cache(message_id TEXT PK REFERENCES messages ON DELETE CASCADE, text_body TEXT, html_body TEXT NULL)` — reconstructible.
- `attachments(id TEXT PK, message_id TEXT REFERENCES messages ON DELETE CASCADE, part_index INTEGER, filename TEXT NULL, mime_type TEXT, size INTEGER, content_id TEXT NULL, inline INTEGER, cache_path TEXT NULL)`
- `threads(id TEXT PK, account_id TEXT REFERENCES accounts ON DELETE CASCADE, subject TEXT, root_key TEXT, UNIQUE(account_id, root_key))` — dérivée (D-15).
- `messages_fts` : `fts5(message_id UNINDEXED, subject, addresses, body, tokenize = 'unicode61 remove_diacritics 2')`.

Famille **intentions** (jamais jetée) :
- `drafts(id TEXT PK, account_id TEXT REFERENCES accounts ON DELETE CASCADE, identity_id TEXT, to_json, cc_json, bcc_json TEXT, subject TEXT, body_text TEXT, body_html TEXT NULL, in_reply_to TEXT NULL, state_json TEXT, updated_at INTEGER)`
- `draft_attachments(draft_id TEXT REFERENCES drafts ON DELETE CASCADE, position INTEGER, filename TEXT, mime_type TEXT, data BLOB, PRIMARY KEY(draft_id, position))`
- `actions(seq INTEGER PRIMARY KEY AUTOINCREMENT, id TEXT UNIQUE, account_id TEXT REFERENCES accounts ON DELETE CASCADE, kind_json TEXT, targets_json TEXT, server_targets_json TEXT, idempotency_key TEXT, state TEXT, attempts INTEGER, last_error TEXT NULL, created_at INTEGER, not_before INTEGER, undo_json TEXT NULL, UNIQUE(account_id, idempotency_key))` — `seq` donne l'ordre de rejeu (invariant 19) ; `server_targets_json` permet de retrouver la cible après reconstruction du miroir (8.4).

Événements :
- `events(seq INTEGER PRIMARY KEY AUTOINCREMENT, at INTEGER, account_id TEXT NULL, category TEXT, kind_json TEXT)` — rétention bornée (on garde les 10 000 derniers).

Règles de stockage :
- Aucune colonne ne contient de secret (invariant 15) : un test parcourt toutes les tables après un scénario avec mot de passe et jetons et n'y trouve aucune valeur secrète.
- « Jeter le miroir » d'un compte supprime ses lignes de `mailboxes`, `messages` (et en cascade apparitions, sources, cache, pièces jointes, server_ids), `threads`, et sa part de `messages_fts`, sans toucher `accounts`, `identities`, `drafts`, `draft_attachments`, `actions` (invariant 16).
- Pagination : curseur opaque encodant la position de tri (`sort_date`, `id`) ; jamais d'`OFFSET` ; limite bornée par `PageRequest::bounded_limit` (invariant 29).
- Accès : `Database` possède une `rusqlite::Connection` dans un `std::sync::Mutex` (un seul écrivain, D-27) ; les requêtes sont courtes. Les dépôts exposent des fonctions typées (pas de SQL hors de `storage/`).

## 5. Contrat d'API (`api/`)

- `Request` : énumération sérialisable, une variante par opération v1 du §12.4 ; `Response` : énumération des réponses. `Engine::execute(Request) -> Result<Response, MiraError>` route par un `match` exhaustif vers **un handler par opération** (`api/handlers/<domaine>/<operation>.rs`). Méthodes typées supplémentaires facultatives.
- Événements : `Engine::subscribe(EventFilter) -> EventStream` (rattrapage depuis la base à partir de `after`, puis diffusion en direct).
- Le CLI ne fait que traduire ses sous-commandes en `Request` et afficher la `Response` (humaine ou `--json`).

## 6. Sécurité réseau

- `Security::Plaintext` n'est accepté que si l'hôte est une adresse de boucle locale (`127.0.0.1`, `::1`, `localhost`) ; sinon `MiraError` `tls.plaintext_refused` (AG-03). C'est ce qui permet de tester contre le testkit sans TLS, sans jamais affaiblir la production.
- TLS : `rustls` avec les racines du système ou `webpki-roots` (choix du module `imap/` et `smtp/`).
