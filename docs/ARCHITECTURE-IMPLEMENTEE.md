# Architecture implémentée — correspondance décisions → code

État au 2026-10-14, branche `feat/E-01-engine-overnight`, dernier commit `34ef005`. Tout ce qui suit a été vérifié par lecture du code ou par commande (comptages, `grep`). Les chemins sont relatifs à `engine/crates/` ; **`C/`** abrège `mira-core/src/`. La référence (`docs/MIRA-architecture-reference.md`) reste l'autorité ; les décisions d'implémentation sont dans `docs/DECISIONS-AGENT.md` (AG-xx) ; les écarts proposés à la référence dans `docs/PROPOSITIONS-REFERENCE.md` (PR-xx).

Ce document décrit ce qui **existe**. Quand une décision n'a pas de code, ou qu'un comportement diffère de la référence, c'est écrit.

> **Mise à jour du 2026-10-21 :** le CLI (`mira-cli`) et le TUI (`mira-tui`) ont été retirés du workspace (AG-63, PR-19) ; le seul client du moteur est l'application Flutter, par le pont FFI (`bridge/`). Les passages des §5, §6 et §10 qui citent le CLI (commandes `mira …`, codes de sortie, `--wait`, tests `mira-cli/tests/`) décrivent l'état du 2026-10-14 : chaque commande citée correspond à la requête `Request` du même nom, que le pont envoie aujourd'hui. Les scénarios S1–S14 restent testés au niveau du moteur (`mira-core/tests`).

## 1. Vue d'ensemble

Workspace Cargo `engine/` (toolchain `1.99.0` épinglée, édition 2024, resolver 3). Neuf crates ; environ 38 600 lignes de Rust hors dossiers `tests/` ; 1 433 fichiers `.rs` ; 1 080 tests réussis, 1 ignoré (voir §10).

| Crate | Rôle | Dépend de | Tests (attributs `#[test]`) |
|---|---|---|---|
| `mira-vocabulary` | Types du domaine, erreurs à code stable, événements, version du contrat, interface `Provider` | `serde`, `thiserror` seulement (A1) | 32 |
| `mira-core` | Le moteur : 26 modules de premier niveau (§2) | `mira-vocabulary` + bibliothèques externes encapsulées | 716 |
| `mira-provider-gmail`, `-outlook`, `-yahoo`, `-generic` | Descriptions pures de services (données du §6.7, **non vérifiées en ligne**) | `mira-vocabulary` seulement (A2) | 13 / 8 / 8 / 4 |
| `mira-assembly` | Composition root : `build_registry()`, `production_config(profil)`, réexports pour les clients (pont Flutter) | tout | 4 |
| `mira-testkit` | Faux serveurs IMAP / SMTP / OAuth2 sur `127.0.0.1` ; boîte noire | aucun crate `mira-*` (A5) | 105 |

Sens des dépendances : pont FFI (`bridge/`, hors workspace) → `mira-assembly` → `mira-core` et providers → `mira-vocabulary`. Le cœur ne nomme aucun provider ; seul `mira-assembly/src/composition.rs` les connaît tous (une ligne par provider, repli `generic` en premier).

Données d'un profil (`C/profile/paths.rs`) : `$MIRA_HOME/profiles/<profil>/`, sinon `$XDG_DATA_HOME/mira/profiles/<profil>/`, sinon `~/.local/share/mira/profiles/<profil>/`, droits `0700`. Contenu : `mira.sqlite3` (WAL), `engine.lock` (verrou exclusif D-27), `attachments/` (cache des pièces jointes), `tmp/`. Un nom de profil vide, `.`, `..` ou contenant `/`, `\`, NUL est refusé.

## 2. Les 26 modules de `mira-core/src`

| Module | Responsabilité | Bibliothèque encapsulée | Fichiers / sous-modules notables |
|---|---|---|---|
| `accounts/` | Cycle de vie des comptes : recherche, choix du provider et des serveurs (`NetworkPolicy` > serveurs saisis > provider), construction, test de connexion, suppression et effacement des traces | — | `new_account.rs`, `servers.rs`, `connection_check.rs`, `deletion.rs` (`erase_account`, `finish_interrupted_deletions`), `network_policy.rs` (`BackgroundSync { Off, On }`) |
| `actions/` | File d'actions locale : soumission idempotente, effet local optimiste + inverse + cibles serveur, annulation, relance, rebase, **rejeu vers le serveur** | — | `submit.rs`, `queue.rs`, `plan/` (intention → étapes), `steps/`, `effect.rs`, `rebase.rs`, `revert.rs`, `retry.rs`, `cancel.rs`, `batch.rs`, `resolve/` (D-17), `targets/`, `replay/` (`run.rs`, `attempt.rs`, `classify.rs`, `settle.rs`, `recover.rs`, `execute/`) |
| `api/` | Contrat : `Request`/`Response`, `Engine`, un handler par opération | — | `engine.rs`, `router.rs` (`match` exhaustif par domaine), `contract/`, `handlers/<domaine>/<opération>.rs`, `startup.rs`, `config.rs` (`EngineConfig`) |
| `auth/` | OAuth2 code + PKCE avec écouteur en boucle locale, jetons, rafraîchissement sérialisé par compte, coffre d'identifiants | `reqwest`, `sha2`, `base64`, `rand` | `flow/`, `flow_table/` (flux en cours, TTL 300 s), `tokens/`, `renewal.rs`, `vault/`, `client_config.rs` (`OAuthClientConfig`), `loopback.rs` |
| `body_storage/` | Rangement unique d'un corps analysé dans le miroir, partagé par la synchro et le téléchargement à la demande | — | `store.rs`, `attachments.rs` |
| `capabilities/` | Capacités annoncées par le serveur moins exceptions du provider ; choix des stratégies | — | `parse.rs`, `effective.rs`, `strategy.rs`, `server_capability.rs` |
| `drafts/` | Brouillons (neuf, réponse, transfert), édition, pièces jointes (≤ 25 Mo), envoi planifié et annulation | — | `create/`, `compose/`, `attachments/`, `scheduling.rs`, `threading.rs`, `identity.rs` |
| `events/` | Bus d'événements numérotés : écrits en base puis diffusés ; abonnement reprenable | `tokio` (`broadcast`) | `bus.rs`, `recording.rs`, `subscription.rs`, `sink.rs` |
| `fetching/` | Seul lien réseau de la lecture : corps, pièces jointes, recherche serveur, à la demande, en tâche longue | — | `access.rs` (`ImapAccess`), `body.rs`, `attachment.rs`, `atomic_write.rs`, `server_search.rs`, `locate.rs`, `ingest.rs` |
| `html/` | Assainissement HTML, ressources distantes bloquées et listées (liste blanche) | `ammonia` | `sanitize.rs`, `remote.rs` |
| `imap/` | Client IMAP **maison** de haut niveau (`async-imap` a été retiré, AG-29) : connexion TLS/STARTTLS, LIST, SELECT/QRESYNC, FETCH, STORE, MOVE/COPY/EXPUNGE, APPEND, libellés, recherche, IDLE | `rustls`, `tokio-rustls`, `webpki-roots` | `connection/`, `protocol/` (lecteur borné), `parse/`, `operations/`, `types/`, `errors/` |
| `mailboxes/` | Rôles de dossiers (SPECIAL-USE d'abord, puis noms usuels) | — | `special_use.rs`, `common_names.rs`, `role.rs` |
| `maintenance/` | Purge du cache, diagnostic, export de profil (D-29), abandon d'un miroir avec ses fichiers | — | `profile_export.rs`, `diagnostic.rs`, `cache_purge.rs`, `account_cache.rs`, `export_file.rs` |
| `mime/` | Analyse tolérante, ne rend jamais d'erreur ; noms de pièces jointes assainis | `mail-parser` | `message.rs` (plafond 50 Mo), `bodies.rs`, `attachments.rs`, `attachment_name.rs`, `html_text.rs` |
| `operations/` | Tâches longues : identifiant, progression par événements, annulation coopérative, rétention (256 terminées) | — | `registry.rs`, `handle.rs`, `retention.rs` |
| `ports/` | Dépendances injectables : horloge, identifiants, trousseau | `uuid`, `keyring` | `clock.rs`, `ids.rs`, `secrets/` (`KeyringSecretStore`, `MemorySecretStore`, `SecretValue` masqué) |
| `profile/` | Emplacements, verrou exclusif | `std::fs` (`File::try_lock`) | `paths.rs`, `lock.rs`, `environment.rs` |
| `reading/` | Lectures **locales** (invariant 27) : listes par vue, messages, corps, pièces jointes, recherche | — | `list.rs`, `query.rs`, `body.rs`, `search.rs`, `counters.rs` |
| `registry/` | Registre des providers, détection, repli | — | `provider_registry.rs`, `resolve.rs` |
| `replayer/` | Rejeu de fond : une tâche par compte (récupération puis `replay_actions`), verrous par compte | — | `account_task.rs`, `supervisor.rs`, `gates.rs`, `credentials.rs`, `work.rs` |
| `send/` | Envoi d'un brouillon : MIME, SMTP, copie dans Envoyés, reprise d'un envoi interrompu | `mail-builder` | `execute.rs`, `deliver.rs`, `settle.rs`, `resolve.rs`, `sent_copy.rs`, `draft_copy.rs`, `message_id.rs` |
| `smtp/` | Composition MIME et client d'envoi ; erreurs classées | `lettre` | `compose/`, `send.rs`, `transport.rs`, `errors.rs`, `reply_codes.rs` |
| `storage/` | SQLite par profil : `Database`, transactions `Tx`, migrations, un dépôt par entité, pagination par curseur, introspection, effacement | `rusqlite`, `zstd` | `database.rs`, `tx.rs`, `migrations/`, `repositories/`, `pagination/`, `erasure.rs` |
| `sync/` | Synchronisation descendante (ch. 10) : descente par étapes, stratégies, identité, renumérotation, plage, temps réel, supervision | — | `download/`, `strategy/`, `identity/`, `resync/`, `window/`, `realtime/`, `supervisor/`, `session/` (budgets), `report/`, `requests/` |
| `sync_service/` | Service de synchro du moteur : superviseur lancé à l'ouverture, piloté par les comptes et la façade | — | `service.rs`, `accounts.rs`, `requests.rs`, `credentials.rs` |
| `threads/` | Fils calculés par MIRA, recalculables | — | `key.rs`, `assign.rs`, `recompute.rs`, `subject.rs`, `read.rs` |

Les modules `body_storage`, `replayer` et `sync_service` sont privés (`mod`, pas `pub mod`) ; les 23 autres sont publics.

## 3. Correspondance D-01 … D-34 → code

| ID | Décision | Où dans le code | Statut / tests |
|---|---|---|---|
| D-01 | Rust | tout le workspace ; `engine/rust-toolchain.toml` (1.99.0) | fait |
| D-02 | Pas de plugins dynamiques | liaison statique ; `engine/deny.toml` interdit `libloading`, `openssl` | `tools/check-architecture` A7, `tools/check-forbidden-patterns` |
| D-03 | Trois blocs cœur / providers / assemblage | `mira-core` (+ `mira-vocabulary`), `mira-provider-*`, `mira-assembly` | A2–A4 |
| D-04 | CLI d'abord | `mira-cli/src/lib.rs` (`run(args, io, deps)`), `main.rs` ; le CLI traduit des sous-commandes en `Request` | `mira-cli/tests/` (191 tests) |
| D-05 | Provider = profil + particularités | `mira-vocabulary/src/provider/` (`Provider`, `ProviderDescriptor`, `ProviderQuirks`, `ProviderLimits`) ; `mira-provider-*/src/{descriptor,provider,servers}.rs`, plus `roles.rs` (gmail, outlook, yahoo) et `translate.rs` (gmail) | tests par tables de chaque provider |
| D-06 | Le cœur ne nomme aucun provider | `mira-core` ; faux providers de tests dans `mira-core/tests/provider_fakes/` | A3, A6 (tests du cœur compris) |
| D-07 | Providers purs | structures unitaires, dépendance unique `mira-vocabulary` | A2 |
| D-08 | Capacités = annonce du serveur − exceptions | `C/capabilities/` (`parse_capabilities`, `EffectiveCapabilities::compute`, `choose_*`) | `capabilities_*.rs` |
| D-09, D-10 | Un profil par session, données et secrets par profil | `C/profile/` ; `SecretRef::storage_key` = `mira/<profil>/<compte>/<nature>` (`mira-vocabulary/src/domain/auth.rs`) | `profile_*.rs`, `ports_secrets*.rs` |
| D-11 | Réglages d'application / de profil | **non implémenté** : seuls les réglages par compte existent (`SyncSettings` : plage, corps préchargés, boîtes synchronisées) | — |
| D-12 | Message unique, boîtes = vues | tables `messages` + `appearances` (`v001_initial.sql`) ; `C/storage/repositories/{messages,appearances,views}/` ; `C/reading/` | `storage_messages.rs`, `reading_*.rs`, `invariant_8_*`, `invariant_9_*` |
| D-13 | Identifiant interne + correspondance, pas de fusion en cas de doute | table `server_ids` ; `C/storage/repositories/server_ids/` ; `C/sync/identity/` (`recognize.rs`, `create.rs`, `placement.rs`) | `invariant_7_*`, `invariant_23_*`, proptest 7–12 |
| D-14 | Drapeaux au niveau du message | colonnes `messages.seen/flagged/answered/forwarded`, `appearances.deleted` ; `C/sync/identity/flags.rs` | `invariant_12_*` |
| D-15 | Fils calculés par MIRA | `C/threads/` ; table `threads` dérivée | `threads_*.rs`, `invariant_13_*` |
| D-16 | Source brute zstd, analyse en cache | `C/storage/repositories/{raw_sources,parsed_cache}/` ; `C/mime/` ; `C/body_storage/` | `invariant_14_*`, `mime_*.rs` |
| D-17 | Second libellé sur compte à dossiers refusé | `C/actions/resolve/messages.rs` (`add_to`) → `incompatible_state.multiple_labels_unsupported` | `actions_effects_labels.rs`, `api_write_mailboxes.rs` |
| D-18 | Trois familles de données | schéma en trois sections ; `C/storage/repositories/mirror/discard.rs` ; `C/maintenance/account_cache.rs` (`discard_mirror_and_cache`) | `invariant_16_*`, `maintenance_discard_cache.rs` |
| D-19 | Entités | `mira-vocabulary/src/domain/` | tests du vocabulaire |
| D-20 | Descente par étapes | `C/sync/download/` : `mailbox_list.rs`, `order.rs` (réception d'abord), `account.rs`, `prefetch.rs`, `orphans.rs` | `api_sync_first.rs`, `scenario_s4_*` |
| D-21 | Plage de 90 jours, corps préchargés, remonter | `SyncSettings` (90 j, 50 corps) ; `C/sync/window/extend.rs` (`extend_window`, tâche longue annulable) ; préchargement ≤ 10 Mo par message | `api_sync_window.rs`, `scenario_s12_*` |
| D-22 | Détection des changements d'après les capacités | `C/capabilities/strategy.rs` ; `C/sync/strategy/` (`plan.rs`, `changes.rs`, `first.rs`) ; QRESYNC → CONDSTORE → comparaison complète | `capabilities_strategy.rs`, `imap_qresync.rs`, 4 jeux de capacités du testkit |
| D-23 | Le serveur gagne, actions locales rejouées par-dessus | `C/actions/rebase.rs` (`reapply_pending_local_effects`), appelé par `C/sync/strategy/finish.rs` dans la même transaction que les compteurs et le repère | `actions_rebase.rs`, `invariant_21_*`, `scenario_s13_*` |
| D-24 | Optimisation « Tous les messages » reportée | non implémentée (reportée par la référence) | — |
| D-25 | Écritures optimistes, file d'actions | `C/actions/` (soumission, effet local, inverse), `C/actions/replay/` (rejeu), `C/replayer/` (tâche de fond par compte), `C/drafts/` + `C/send/` ; table `actions` (`storage/repositories/actions/`) | `invariant_17_*`, `18_*`, `19_*`, `22_*`, `28_*`, `actions_replay_*.rs` |
| D-26 | Contrat d'API | `C/api/` : `Engine::open/execute/subscribe/shutdown/shutdown_within`, 16 domaines, 71 variantes, un handler par opération | `contract_snapshot_*.rs`, `snapshots/contract_v1.json` |
| D-27 | Verrou de profil | `C/profile/lock.rs` (`File::try_lock`, `incompatible_state.profile_locked`), pris par `Engine::open` | `profile_lock.rs`, `api_engine.rs` |
| D-28 | Découpage v1 | décision de départ 3 : colonne « v1 » du §12.4 ; la façade expose 71 variantes de `Request` (J8) | `invariant_6_*` (CLI = façade) |
| D-29 | Export du profil | `C/maintenance/profile_export.rs` (JSON `format_version: 1` : comptes, identités, autorisations, brouillons, actions ; sans secret ni miroir) ; handler `C/api/handlers/maintenance/profile_export.rs` ; CLI `mira maintenance export-profile` ; fichier `0600` créé en exclusif | `maintenance_export_file.rs`, `api_read_maintenance.rs`. Import : reporté (décision de départ 4) |
| D-30 | Façade bibliothèque + CLI | `Engine` async (`mira_core::api`) ; processus séparé reporté | — |
| D-31 | Pile du brief | `tokio`, `mail-parser`, `rusqlite` (bundled, FTS5), `keyring` 4, `lettre`, `reqwest` (rustls), `ammonia`, `zstd` ; `sqlite-vec` reporté ; `flutter_rust_bridge` hors périmètre | `engine/Cargo.toml` |
| D-32 | Taille cible (10 comptes × 100 000 messages, page < 50 ms) | `reading_performance.rs` mesure **20 000** messages ; la mesure à 100 000 n'est **pas** faite (reste de J10) | partiel |
| D-33 | Vocabulaire distinct | crate `mira-vocabulary` | A1 |
| D-34 | Sous-domaines = modules, bibliothèques encapsulées | §2 ci-dessus | A8 |

## 4. Stockage

Base `mira.sqlite3` par profil. Réglages à l'ouverture (`C/storage/database.rs`) : `journal_mode=WAL`, `foreign_keys=ON`, `synchronous=NORMAL`, `secure_delete=ON`. Un seul écrivain (`Mutex<Connection>` ; `write(|tx| …)` / `read(|tx| …)`, AG-05).

Migrations (`C/storage/migrations/`, suivies par `PRAGMA user_version`, appliquées dans une transaction) :

| Version | Contenu |
|---|---|
| v001 | schéma initial : `accounts`, `identities`, `remote_content_allowances`, `mailboxes`, `threads`, `messages`, `appearances`, `server_ids`, `raw_sources`, `parsed_cache`, `attachments`, `messages_fts` (FTS5), `drafts`, `draft_attachments`, `actions`, `events` |
| v002 | `accounts.profile_id` ; index unique « une seule identité par défaut par compte » |
| v003 | `account_token_meta` (expiration des jetons ; aucun jeton) |
| v004 | index de lecture (`messages_by_date`, non lus, suivis, `mailboxes_by_account_role`) |
| v005 | `account_idempotency` (clé d'idempotence de `AddAccount`) |
| v006 | `rules` (définition JSON, position, `scope_all`), `rule_accounts` (portée, cascade à la suppression du compte), `rule_pending` (messages à évaluer, reprise sur crash) — AG-64, AG-65 |
| v007 | `rule_idempotency` (clés d'idempotence des écritures de règles, avec empreinte ; non purgées) — AG-65 |
| v008 | `local_labels` (nom unique sans casse, couleur), `thread_local_labels` (pose par `(compte, clé du fil)`, cascade à la suppression du libellé ou du compte, sans clé étrangère vers `threads`) — AG-72 |
| v009 | `local_label_idempotency` (clés d'idempotence des libellés locaux, avec empreinte et résultat JSON ; non purgées) — AG-73 |

Trois familles (D-18) : **configuration** (`accounts`, `identities`, `remote_content_allowances`, `account_token_meta`, `account_idempotency`, `rules`, `rule_accounts`, `rule_idempotency`), **miroir** jetable (`mailboxes`, `messages`, `appearances`, `server_ids`, `raw_sources`, `parsed_cache`, `attachments`, `threads`, `messages_fts`), **intentions** jamais jetées (`drafts`, `draft_attachments`, `actions`, `rule_pending`). `events` est à part. Aucune colonne ne contient de secret (invariant 15, testé en parcourant toutes les tables).

Points d'implémentation à connaître :
- Pagination par curseur opaque `mira1|<date>|<id>` encodé en hexadécimal, jamais d'`OFFSET` (invariant 29) ; curseur invalide → `invalid_input.cursor`.
- Apparition provisoire (déplacement ou libellé hors ligne) : UID stocké très négatif, lu `0`, `needs_revalidation` ; pendant une revalidation, les UID passent en négatif pour éviter la collision `UNIQUE(mailbox_id, uid)` (AG-13, AG-16, PR-06).
- Recherche : FTS5 échappé, ET implicite mot par mot, tri par date et non par pertinence (pour la pagination par curseur).
- Rétention des événements : 30 jours et 100 000 lignes, appliquée à l'ouverture du moteur et après chaque passe de synchro (`C/storage/repositories/events/prune.rs`, AG-28).
- Effacement : `Database::erase_traces()` (`C/storage/erasure.rs`) fusionne l'index FTS5 (`optimize`) puis `wal_checkpoint(TRUNCATE)` ; appelé par `erase_account`. Pas de `VACUUM` (AG-33).

## 5. Contrat d'API, événements, erreurs, CLI

**Requêtes** (`C/api/contract/`). `Request` = 16 domaines et **71 variantes** (comptées dans les énumérations) :

| Domaine | Variantes | Domaine | Variantes |
|---|---|---|---|
| Engine | 2 | Search | 3 |
| Provider | 2 | Maintenance | 6 |
| Account | 8 | Auth | 5 |
| Identity | 4 | Operation | 2 |
| Mailbox | 2 | Event | 1 |
| Message | 9 | Action | 6 |
| Thread | 2 | Compose | 11 |
| Queue | 3 | Sync | 5 |

`Engine::execute(Request) -> Result<Response, MiraError>` route par `match` exhaustif (`api/router.rs`) vers un handler par opération (`api/handlers/<domaine>/<opération>.rs`). Les énumérations sont sérialisables : même forme pour le CLI `--json`, Flutter ou un futur transport. Version du contrat `1.0` (`mira-vocabulary/src/contract/version.rs`), vérifiée à l'ouverture (invariant 33). Le test de contrat compare des instantanés JSON versionnés de chaque variante (`mira-core/tests/contract_snapshot_*.rs`, `snapshots/contract_v1.json`, exemples dans `contract_examples/`).

**Événements** (`mira-vocabulary/src/events/kind.rs`) : `EventKind` compte **42** variantes (et non 44 comme le notaient des documents antérieurs). Sept sont déclarées mais **jamais émises** par le moteur (vérifié par `grep` dans `mira-core/src` et `mira-cli/src`) : `EngineFatalError`, `MailboxUpdated`, `AttachmentProgress`, `ThreadUpdated`, `QuotaFull`, `CertificateNeedsApproval`, `RateLimited`. Numérotation persistée, abonnement reprenable (`Engine::subscribe`, ou `Request::Event(GetEvents)` pour un client sans abonnement comme le CLI).

**Erreurs** : `MiraError` = catégorie + code stable + message + réessayable + action suggérée + contexte ; 11 catégories, codes uniques et préfixés par la catégorie (invariant 31). Codes de sortie du CLI : 0 succès ; 10 entrée invalide, 11 introuvable, 12 état incompatible, 13 réseau, 14 TLS, 15 authentification, 16 serveur, 17 stockage local, 18 trousseau, 19 annulé ; 84 interne (`mira-cli/src/exit_codes.rs`).

**CLI** (`mira-cli/src/commands/<domaine>/`). 72 commandes : engine (2), provider (2), account (9, dont `add-oauth` qui n'est pas une variante de `Request` mais enchaîne StartOAuth, CompleteOAuth, AddAccount), identity (4), auth (5), operation (2), mailbox (6), message (9), thread (2), search (3), maintenance (6), action (2), draft (11), queue (3), sync (5), events (1). Option globale `--json` (enveloppe `{contract_version, ok, data | error}`) et `--profile`. Les secrets ne passent que par `--password-stdin` (jamais en argument). Invariant 6 : `mira-cli/tests/invariant_6_every_request_has_a_cli_command.rs` énumère chaque variante de `Request`.

## 6. Flux principaux

### 6.1 Ajout d'un compte et première synchronisation (S1–S4)
1. `mira account add --address … --password-stdin` (ou `account add-oauth`, un seul processus : URL d'autorisation sur stderr, `CompleteOAuth` borné par `--timeout`) construit `Request::Account(AddAccount)` (`mira-cli/src/commands/account/`).
2. `C/api/handlers/account/add.rs` : verrou d'ajout sérialisé ; si la clé d'idempotence est connue, rend le compte existant (autre adresse → `invalid_input.idempotency_conflict`) ; plan (provider détecté, serveurs) ; le secret est rangé dans le trousseau (`CredentialVault`, jamais en base).
3. `register.rs` : prend un permis du `ConnectionBudget` du compte, teste la connexion IMAP (`connection_check`), puis écrit en **une transaction** le compte, son identité par défaut et la clé d'idempotence. Échec → rien n'est enregistré et les secrets déjà rangés sont supprimés.
4. Événement `AccountAdded`, puis `sync.start_account` : le superviseur (`C/sync/supervisor/`) lance **une tâche par compte** (invariant 24). Le CLI ne lance la synchro de fond que pour `account add|add-oauth --sync-wait` et pour `sync …` (AG-26).
5. Une passe (`C/sync/download/account.rs::run_pass`, événements `SyncStarted`/`SyncProgress`/`SyncFinished|SyncFailed`) : liste des boîtes ; `sync_order` (réception d'abord, puis Envoyés, Brouillons, Archive/Tous/Suivis, autres, Corbeille, Spam) ; pour chaque boîte `sync_mailbox` choisit `Initial`, `Renumbered` ou `Incremental(détection)` ; en-têtes de la plage (90 j) ; pour la réception, **préchargement** des 50 corps les plus récents (≤ 10 Mo chacun) juste après ses en-têtes (AG-31) ; fin de boîte : rebase des actions locales, compteurs, **puis** repère (invariant 20) ; fin de passe complète : suppression des orphelins.
6. Temps réel (`C/sync/realtime/`, `supervisor/`) : IDLE sur une connexion dédiée si le budget laisse un permis de réserve (limite ≥ 3), sinon relevé périodique (délai minimal du provider) ; reconnexion de 1 s à 300 s ; IDLE renouvelé toutes les 25 min.

### 6.2 Lecture hors ligne (S6, invariant 27)
`mira message list|get|body|raw|attachments`, `thread …`, `search local` appellent `C/reading/` : uniquement la base du profil, aucune connexion. Un message à l'état `HeadersOnly` donne `incompatible_state.body_unavailable_offline` (S6). `message fetch`, `save-attachment` et `search server` passent par `C/fetching/` : permis du budget du compte, tâche longue (`Response::Operation`), CLI qui attend (`--timeout`, 120 s). `safe-html` assainit via `C/html/` hors verrou de base.

### 6.3 Écriture → file → rejeu (S7, S8, invariants 17–19, 22, 28)
1. `mira action apply …` → `ApplyAction` : `C/actions/submit.rs`/`queue.rs` écrivent en **une transaction** l'action (état `Pending`, clé d'idempotence), son effet local (étapes idempotentes selon `ProviderQuirks`), l'inverse (`undo_json`) et les cibles serveur ; rend la main sans réseau (invariant 28).
2. `C/replayer/` : une tâche par compte (sonde 250 ms, réveil au submit/retry) ; `C/actions/replay/run.rs::replay_actions` rejoue dans l'ordre de `seq` (invariant 19) : `begin_attempt` écrit `InProgress` **avant** le serveur (invariant 17, conditionnel à `state='pending'` contre l'annulation concurrente), exécute (`replay/execute/` : traduction par capacités, vérification avant rejeu non idempotent), puis `Succeeded` + `ActionApplied`.
3. Échec (`classify.rs`) : passager (réseau, TLS, stockage local, serveur réessayable) → l'action reste en file avec attente croissante ; authentification → compte `AuthRequired`, file conservée (S10) ; définitif → `revert_local_effect`, état `Failed`, `ActionFailed` (invariant 22). Le rejeu s'arrête au premier échec passager ou d'authentification.
4. Le CLI vit le temps d'une commande : sans `--wait`, l'action reste en file ; avec `--wait` (60 s) il attend la fin de l'action : `Failed` → 16, `Uncertain` ou délai → 13, compte à réauthentifier → 15 immédiat (AG-24, AG-33).
5. Côté synchro, la valeur du serveur gagne puis `reapply_pending_local_effects` rejoue l'intention locale en attente (D-23).

### 6.4 Envoi, plantage et issue inconnue (S9, invariants 18, 26)
1. `mira draft send` → `schedule_send` (`C/drafts/scheduling.rs`) crée l'action `Send` (délai d'annulation `SEND_UNDO_DELAY_MILLIS` = 10 s, `C/actions/model.rs` ; `cancel-send` tant qu'elle est `Pending`).
2. Au rejeu, `begin_attempt` passe l'action à `InProgress` **et** mémorise le `Message-ID` dans `ServerTargets.sent_message_id` (`prepare_send_attempt`, `C/send/message_id.rs`) dans la même transaction, avant tout SMTP ; `C/send/execute.rs` compose le MIME avec ce `Message-ID` et appelle `deliver` (`lettre`, délai global 120 s).
3. Après acceptation par le serveur, plus aucune erreur ne remonte : événement `SendDeparted`, copie dans Envoyés (selon `server_saves_sent_copy`), purge de la copie serveur du brouillon, suppression du brouillon — chaque échec est journalisé (`tracing::warn`) sans jamais provoquer de renvoi.
4. Erreur pendant l'envoi : `send_outcome_is_unknown` (`C/actions/replay/classify.rs`) = catégorie `Network` autre que `network.unreachable` → `settle_interrupted_send` cherche le `Message-ID` dans Envoyés : trouvé → `Succeeded` ; sinon `Uncertain` + `SendUncertain`. Seul `network.unreachable` (connexion TCP impossible, rien n'est parti) est passager.
5. Plantage du processus : au premier tour de la tâche de rejeu du compte après l'ouverture, `recover_interrupted_actions` règle chaque action restée `InProgress` : envoi → même vérification dans Envoyés (Envoyés illisible : reste `InProgress`) ; autre action → remise en file. Jamais de renvoi automatique.
6. `mira draft resolve-send` → `resolve_uncertain_send` (`C/send/resolve.rs`) : renvoyer (nouvelle tentative, **nouveau** `Message-ID`) ou abandonner (le brouillon revient en édition).

### 6.5 Suppression d'un compte et effacement des traces (S14)
`handle_delete_account` (`C/api/handlers/account/delete.rs`) : marque le compte « en suppression » (`deleting=1`), arrête et **attend** sa tâche de synchro et sa tâche de rejeu (verrou par compte `replayer/gates.rs`, 30 s), abandonne ses flux OAuth, puis `erase_account` (`C/accounts/deletion.rs`) : fichiers de cache des pièces jointes (chemins sous `attachments/`, sans `..`), secrets du trousseau, lignes de la base (cascade), `erase_traces()` (fusion FTS5 + troncature du WAL, `secure_delete=ON`), événement `AccountDeleted`. Le serveur n'est jamais contacté. Une suppression interrompue par un arrêt brutal est **achevée à l'ouverture** (`finish_interrupted_deletions`). Les autres comptes ne sont pas touchés.

## 7. Invariants et scénarios

Les invariants 1–33 et les scénarios S1–S14 sont reliés à leurs tests nommés dans **`docs/ETAT/INVARIANTS.md`**, régénéré par `tools/update-invariants-table` à partir des fonctions `invariant_<n>_…` et `scenario_s<n>_…`. Les invariants 1–5 sont tenus par les garde-fous (§8) ; les autres par des tests (`mira-core`, `mira-vocabulary`, `mira-cli`). Les 14 scénarios sont exécutés **via le CLI en processus** contre le testkit (`mira-cli/tests/scenario_s01_….rs` à `scenario_s14_….rs`) et, pour la plupart, aussi au niveau du moteur. La revue « un par un » des 33 invariants (au-delà de l'existence d'un test) n'est pas faite (J10).

## 8. Garde-fous mécaniques

`./ci.sh` (ou `make check`) enchaîne :

| Contrôle | Ce qu'il vérifie |
|---|---|
| `tools/test-check-norms`, `tools/test-check-architecture` | auto-tests des garde-fous (fixtures `tools/fixtures/{ok,bad}`, workspace saboté) |
| `tools/check-norms` | N1 fonction ≤ 30 lignes ; N2 ≤ 6 fonctions avec corps par fichier (tests compris) ; N3 noms `utils`/`helpers`/`common`/`misc` ; N4 aucun commentaire dans un corps ; N5 exactement une ligne `///` avant chaque `fn` (dernier relevé : 1 433 fichiers, 0 violation) |
| `tools/check-architecture` | A0 crates attendus présents ; A1 vocabulaire = `serde` + `thiserror` ; A2 providers = vocabulaire seul ; A3 cœur sans provider ni assemblage ; (A4, règle des clients CLI/TUI, retirée avec eux) ; A5 testkit sans crate `mira-*` ; A6 aucun nom de provider dans le cœur, tests compris ; A7 pas de chargement dynamique ; A8 chaque bibliothèque externe dans son module (table `ENCAPSULATED`) ; A9 aucune notion d'écran |
| `tools/check-forbidden-patterns` | `unwrap`/`expect`/`panic!`/`todo!`/`unimplemented!`/`dbg!`/`unsafe` hors tests ; `TODO` non référencé ; marqueurs de conflit git ; `libloading`/`dlopen` dans un `Cargo.toml` |
| `cargo fmt --check`, `cargo clippy -D warnings` | lints du workspace : `unsafe_code = forbid`, `unwrap_used`/`expect_used`/`panic`/`todo` = deny |
| `cargo test --workspace` | `cargo-nextest` ne s'installe pas (BLOQUÉ-01) ; `ci.sh` bascule sur `cargo test` |
| `cargo deny check bans licenses sources` | pas les avis de sécurité (AG-07) |

Limite des garde-fous : A8 liste encore `async_imap` alors que la dépendance a été retirée (inoffensif) ; `C/imap/mod.rs` porte `#![allow(dead_code)]`.

## 9. Sécurité : ce qui est en place

- Secrets : trousseau système (`keyring`, SecretService) ou faux en mémoire en test ; clé `mira/<profil>/<compte>/<nature>` ; `SecretValue` à `Debug` masqué, sans `Serialize` ; aucun secret en base, journaux, événements, réponses, exports (invariant 15, plusieurs tests).
- Réseau : TLS strict (`rustls`, racines `webpki-roots`, SNI) ; clair refusé hors boucle locale (`tls.plaintext_refused`, AG-03, un seul `is_loopback_host`) ; lecteur IMAP borné (ligne 1 Mo, littéral 64 Mo, réponse 64 Mo, délai d'inactivité réarmé par bloc, AG-28).
- OAuth : PKCE S256, `state` comparé en temps constant, `state` faux → 400 sans tuer le flux, TTL 300 s, réponse de jetons ≤ 1 Mo, rafraîchissement sérialisé par compte, page de confirmation statique (PR-05).
- HTML : `ammonia` + liste blanche (`cid:`, `data:image/*` hors SVG dans les attributs auto-chargés), URL normalisée avant test, `UrlRelative::Deny`, `data:` interdit dans `href` (AG-29).
- Fichiers : profil `0700`, exports `0600` en création exclusive, pièces jointes écrites de façon atomique, noms assainis (`mime/attachment_name.rs`).
- Budget de connexions par compte, partagé par la synchro, le rejeu, les téléchargements, la recherche serveur et les tests de connexion (invariant 25, AG-28) ; au plus 16 demandes explicites en attente par compte.

## 10. Chiffres vérifiés

`target/rapports/tests.log` (dernier `./ci.sh`, commit `34ef005`) : **1 080 tests réussis, 0 échec, 1 ignoré** (`ports_secrets_keyring.rs` : usage du vrai trousseau, manuel, prévu par la décision de départ 18). Nombre d'attributs de test dans les sources : 1 081. Le fichier `docs/ETAT/TESTS.md` n'a pas été remis à jour depuis le 2026-10-11 (il indique encore 478 tests) ; `docs/ETAT/HORLOGE.md` non plus.

## 11. Limites connues et divergences

Écarts à la référence et précisions : `docs/PROPOSITIONS-REFERENCE.md` (PR-01 à PR-17). Limites, avec leur décision :

**Non fait (reste de J10)**
- `html::sanitize_html` est quadratique en profondeur d'imbrication (1 000 niveaux : 68 ms ; 10 000 : 5,3 s ; 100 000 : plus de 10 min). Aucune borne de taille ni de profondeur dans `C/html/`. Un mail hostile bloque la lecture (AG-34).
- UTF-7 modifié (RFC 3501 §5.1.3) **absent** : les noms de boîtes non ASCII ne sont ni encodés ni décodés (`C/imap/protocol/quoting.rs` ne fait que citer) ; la recherche serveur envoie le texte entre guillemets, sans `CHARSET UTF-8` ni littéral (`C/imap/operations/search.rs`). Remarque #11 de la revue, jamais traitée (AG-12, AG-34).
- Fuzz léger des parseurs MIME / HTML / réponses IMAP : non fait (proptest « ne panique jamais » sur le MIME seulement). Performance à 100 000 messages : non mesurée. Revue des invariants un par un : non faite.

**Comportements limités**
- Synchro (AG-21, AG-25, AG-28) : orphelins supprimés seulement en fin de passe complète ; boîte de destination traitée avant la source (désarchivage vers la réception) : crée encore un nouveau message ; deux copies identiques à ±60 s tenues pour la même lors d'un déplacement ; un message arrivé entre la fin de la première passe et l'établissement d'IDLE n'est vu qu'au relevé suivant ; `extend_window` s'exécute encore en parallèle de la tâche du compte et son annulation n'est testée qu'avant le premier lot ; le repli VANISHED (> 1 M d'UID) n'a pas de test d'intégration.
- CLI de courte durée (AG-19, AG-24, AG-26, AG-31, AG-32) : aucune écoute IDLE au-delà d'une commande ; `search server-result` ne retrouve rien d'un autre processus ; `operation cancel` pendant `extend-window` et suppression de compte pendant une synchro non testables en processus (moteur exclusif par profil) ; `auth reauthenticate` suivi de `oauth-complete` dans deux processus impossible (table des flux en mémoire) ; `action apply --wait` sur un compte `auth_required` sort en 15 depuis AG-33.
- Envoi (AG-30) : `map_io_failure` distingue « injoignable » de « session perdue » par le libellé d'erreur de `lettre` (`"Connection error"`), donc fragile à une montée de version ; une erreur de catégorie `Tls` en cours d'envoi est classée passagère (pas incertaine).
- Identifiants (AG-27, AG-14, AG-22) : `ImapAccess` utilise le jeton rangé sans rafraîchissement automatique (fournisseur d'identifiants non unifié) ; le test de connexion à l'ajout est IMAP seulement (pas SMTP) ; `UpdateAccount` avec d'autres serveurs teste avec le secret rangé (défense complète = changement de contrat).
- Frontière (AG-29) : `X-GM-EXT-1` (`C/imap/operations/labels.rs`) et `\Inbox` (`C/actions/replay/execute/labels.rs`) sont codés en dur dans le cœur au lieu de venir des particularités du provider ; `zeroize` non appliqué (secrets non effacés de la mémoire) ; mutex `std` de la base tenu sur des threads tokio hors trousseau, `GetSafeHtml` et `CheckIntegrity`.
- Miroir : `discard_mirror_and_cache` (`C/maintenance/account_cache.rs`) supprime les fichiers de cache puis la base **sans** appeler `erase_traces()`, contrairement à `erase_account` ; `Database::discard_mirror_of` (qui l'appelle) n'a pas d'appelant en production ; aucune requête de la façade n'expose « jeter le miroir » (seul l'invariant 16 le teste). Les traces d'une base créée avant AG-33 ne sont pas purgées rétroactivement.
- Événements : sept variantes jamais émises (§5). Pas de `RateLimited` bien que le refus `server.rate_limited` existe.
- Production : `production_config` construit un `OAuthClientConfig` **vide** (aucun `client_id` par provider) : `mira account add-oauth` est refusé hors configuration injectée. Les valeurs des providers (serveurs, adresses OAuth, plafonds) viennent du §6.7 et n'ont pas été vérifiées en ligne (`docs/SOURCES.md` est vide). Aucun test ne réussit une poignée de main TLS ni STARTTLS : le testkit parle en clair (AG-03, AG-10).
- Réglages D-11 et optimisation D-24 : non implémentés.
