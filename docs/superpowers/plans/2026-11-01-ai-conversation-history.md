# Historique des conversations IA — plan d'implémentation

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Enregistrer, lister, reprendre, renommer et supprimer les conversations avec l'assistant IA, avec un interrupteur « Conserver l'historique » activé par défaut.

**Architecture:** Le moteur Rust stocke un document par conversation (métadonnées en colonnes, payload texte JSON opaque) dans une table `ai_conversations` (migration v020) et l'expose par huit requêtes du contrat 1.4. L'app Flutter sérialise le fil visible, sauvegarde à chaque tour terminé, et affiche une vue « Historique » qui remplace le fil dans le volet assistant.

**Tech Stack:** Rust 1.99 (édition 2024, `rusqlite`, `serde`), Flutter/Dart (`provider`, `ChangeNotifier`), pont `flutter_rust_bridge` inchangé.

**Spec:** `docs/superpowers/specs/2026-11-01-ai-conversation-history-design.md`

## Global Constraints

- Branche `feat/ai-conversation-history` (déjà créée depuis `dev`). **Ne jamais ajouter `app/lib/ui/ai_connection_card.dart` à un commit** : modification de l'utilisateur, étrangère à ce chantier. Toujours `git add <fichiers précis>`.
- Commits au nom de l'utilisateur, **sans trailer `Co-Authored-By`**. Un commit par tâche, `feat(<domaine>): …`.
- **Ne jamais lancer `./ci.sh` ni `make check`** sans l'accord explicite de l'utilisateur. Tests ciblés seulement (`cargo test -p … --test …`, `flutter test <fichier>`).
- Normes Rust (`engine/`, vérifiées par `tools/check-norms`) : fonction ≤ 30 lignes ; fichier ≤ 6 fonctions avec corps, tests compris ; **aucun commentaire dans un corps de fonction** ; une seule ligne `///` en français avant chaque `fn` ; `//!` de 1 à 3 lignes en tête de fichier ; interdits `utils.rs`, `helpers.rs`, `common.rs`, `misc.rs` ; pas de `unwrap`/`expect`/`panic!` hors tests ; pas de `unsafe` ; un handler par opération, routage par `match` exhaustif ; tests dans des fichiers séparés avec `#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]`.
- Code Dart : identifiants en anglais, `///` en français avant chaque membre public, chaînes d'interface via `t('…')` avec entrée dans `app/lib/i18n/lang/en.dart`.
- Valeurs du spec : titre ≤ **80 caractères** (comptés en caractères, vide ou blanc refusé) ; payload ≤ **2 Mio** (2 097 152 octets), objet JSON ; rétention **200** conversations (éviction de la plus ancienne par `updated_at`, jamais celle qu'on enregistre) ; pages de **50** côté app, `limit` de **1 à 200** côté moteur ; contrat **1.3 → 1.4** ; interrupteur `ai_history_enabled` activé par défaut.
- Codes d'erreur : `invalid_input.ai_conversation_title`, `invalid_input.ai_conversation_too_large`, `invalid_input.ai_conversation_payload`, `incompatible_state.ai_history_disabled`, `not_found.ai_conversation`, et `invalid_input.parameter` pour une `limit` hors bornes.
- Aucun contenu de conversation (titre, payload) dans un événement, un `tracing`, un message d'erreur ni `ai_journal`.
- Le pont FFI et `frb_generated` ne changent pas (vérifié : `execute(request_json)` générique).

## Review Focus

- Titre de plus de 80 caractères composé d'emoji ou d'accents : la troncature se fait aux caractères (jamais au milieu d'un caractère), côté moteur (refus) et côté app (coupe avant l'envoi). Testé en T3 et T8.
- « Tout effacer » pendant qu'une conversation est ouverte : le fil repart à neuf (nouvel `id`), le prochain tour ne ressuscite pas l'ancienne conversation. Testé en T9.
- Interrupteur coupé pendant qu'une réponse est en cours : le tour qui se termine n'est pas enregistré. Testé en T8.
- Conversation dont le payload est illisible (version inconnue ou JSON corrompu) : listée, supprimable, jamais de plantage à l'ouverture. Testé en T6 et T9.
- Échec de `save` (moteur en erreur) : le chat continue, un avertissement discret s'affiche, le tour suivant retente. Testé en T8.

---

### Task 1: Vocabulaire — types et codes d'erreur

**Files:**
- Create: `engine/crates/mira-vocabulary/src/ai/conversation.rs`
- Modify: `engine/crates/mira-vocabulary/src/ai/mod.rs` (déclarer `pub mod conversation;`), `engine/crates/mira-vocabulary/src/lib.rs` (réexporter à côté d'`AiJournalEntry`)
- Modify: `engine/crates/mira-vocabulary/src/error/codes/{invalid_input,incompatible_state,not_found,mod}.rs`
- Test: `engine/crates/mira-vocabulary/tests/ai_conversation.rs`

**Interfaces:**
- Produces (réexportés par `mira_vocabulary`) :
  - `pub struct AiConversationMeta { pub id: String, pub title: String, pub created_at: Timestamp, pub updated_at: Timestamp, pub payload_version: u32 }`
  - `pub struct AiConversation { #[serde(flatten)] pub meta: AiConversationMeta, pub payload: String }`
  - Les deux : `#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]`.
  - Constantes : `invalid_input::{AI_CONVERSATION_TITLE, AI_CONVERSATION_TOO_LARGE, AI_CONVERSATION_PAYLOAD}`, `incompatible_state::AI_HISTORY_DISABLED`, `not_found::AI_CONVERSATION`, chacune enregistrée dans la liste de `codes/mod.rs` (même endroit que `AI_JOURNAL_LIMIT`).

- [ ] **Step 1: Write the failing test** `conversation_round_trips_and_flattens_meta` : sérialiser une `AiConversation` donne un objet JSON dont les clés sont `id`, `title`, `created_at`, `updated_at`, `payload_version`, `payload` (meta aplatie), et la relecture est identique. Second test `new_codes_are_registered` : chaque nouveau code figure dans la liste exposée par `codes/mod.rs` (reprendre la forme du test existant des codes ; sinon `assert_eq!(AI_HISTORY_DISABLED.0, "incompatible_state.ai_history_disabled")` et les quatre autres).
- [ ] **Step 2: Run** `cd engine && cargo test -p mira-vocabulary --test ai_conversation` — Expected: FAIL (types absents).
- [ ] **Step 3: Implement** les types et les cinq codes (libellés `ErrorCode("…")` exactement ceux de Global Constraints).
- [ ] **Step 4: Run** le même test et `cargo test -p mira-vocabulary` — Expected: PASS.
- [ ] **Step 5: Commit** `git add engine/crates/mira-vocabulary && git commit -m "feat(ai): types et codes d'erreur de l'historique des conversations"`

---

### Task 2: Stockage — migration v020 et dépôt `ai_conversations`

**Files:**
- Create: `engine/crates/mira-core/src/storage/migrations/v020_ai_conversations.sql`
- Modify: `engine/crates/mira-core/src/storage/migrations/mod.rs` (ajouter `include_str!("v020_ai_conversations.sql")` à la fin de `MIGRATIONS`), `engine/crates/mira-core/src/storage/repositories/mod.rs` (`pub mod ai_conversations;`)
- Create: `engine/crates/mira-core/src/storage/repositories/ai_conversations/{mod,rows,read,write,history_setting}.rs`
- Modify: tout test existant qui affirme une version de schéma de 19 (`grep -rn "schema_version" engine/crates/*/tests`) → 20
- Test: `engine/crates/mira-core/tests/storage_ai_conversations.rs`, `storage_ai_conversations_trim.rs`, `storage_ai_history_setting.rs` (modèle : `storage_ai_journal.rs`, `Database::open_in_memory()`)

**Interfaces:**
- Consumes: `AiConversationMeta`, `AiConversation`, codes de T1.
- Produces (méthodes de `Tx`, toutes `Result<_, MiraError>`) :
  - `list_ai_conversations(&self, limit: u32, before: Option<&str>) -> Vec<AiConversationMeta>` — ordre `updated_at DESC, rowid DESC`, `before` = `id` de la dernière ligne reçue (même requête à sous-sélection que `list_ai_journal`) ; un `before` inconnu rend une page vide.
  - `get_ai_conversation(&self, id: &str) -> AiConversation` — `not_found.ai_conversation` si absent.
  - `save_ai_conversation(&self, id: &str, title: &str, payload_version: u32, payload: &str, now: i64) -> AiConversationMeta` — upsert : à la création `created_at = updated_at = now` ; à la mise à jour `created_at` est conservé, `updated_at = now`.
  - `rename_ai_conversation(&self, id: &str, title: &str) -> AiConversationMeta` — ne touche pas `updated_at` ; `not_found.ai_conversation` si absent.
  - `delete_ai_conversation(&self, id: &str) -> ()` — `not_found.ai_conversation` si absent.
  - `clear_ai_conversations(&self) -> ()`.
  - `trim_ai_conversations(&self, keep: usize, protect: &str) -> u64` — garde les `keep` plus récentes (par `updated_at DESC, rowid DESC`), ne supprime jamais `protect`, rend le nombre supprimé.
  - `ai_history_enabled(&self) -> bool` (absent = `true`) et `set_ai_history_enabled(&self, enabled: bool) -> ()` — clé `ai_history_enabled`, valeurs `"1"`/`"0"`, dans `engine_settings` ; ne lit ni n'écrit les autres clés.

- [ ] **Step 1: Write the failing tests**
  - `save_twice_keeps_one_row_and_created_at` : deux `save` du même `id` à `now` 10 puis 20 → une ligne, `created_at == 10`, `updated_at == 20`, titre et payload remplacés.
  - `list_orders_by_recency_and_pages_by_before` : trois conversations à 1, 2, 3 → ordre 3,2,1 ; `limit 2` puis `before = dernier id` donne la troisième ; `before` inconnu → vide ; la liste ne contient aucun payload (le type n'en a pas).
  - `get_unknown_is_not_found` (code `not_found.ai_conversation`) et `get_returns_payload_verbatim`.
  - `rename_keeps_updated_at`, `rename_unknown_is_not_found`, `delete_unknown_is_not_found`, `clear_removes_all`.
  - `trim_keeps_newest_and_never_the_protected_one` : 5 lignes, `trim(2, "plus-ancienne")` garde les 2 plus récentes **et** la protégée (3 lignes), rend 2.
  - `history_setting_defaults_to_true_and_is_independent` : défaut `true` ; après `set(false)` → `false` ; `get_engine_settings()` rend les mêmes réglages qu'avant (clé étrangère ignorée) ; `put_engine_settings` ne l'efface pas.
  - `migration_20_applies_on_existing_profile` : copier `migration_16_applies_on_existing_profile` (profil ouvert à la version 19 puis rouvert, table présente, `schema_version() == 20`).
- [ ] **Step 2: Run** `cd engine && cargo test -p mira-core --test storage_ai_conversations --test storage_ai_conversations_trim --test storage_ai_history_setting` — Expected: FAIL (compilation).
- [ ] **Step 3: Implement** la migration (colonnes du spec : `id TEXT PRIMARY KEY, title TEXT NOT NULL, created_at INTEGER NOT NULL, updated_at INTEGER NOT NULL, payload_version INTEGER NOT NULL, payload TEXT NOT NULL` + `CREATE INDEX ai_conversations_by_date ON ai_conversations(updated_at)`) et le dépôt, un fichier par responsabilité (`rows.rs` : constante `COLUMNS` et `meta_from_row`, calqué sur `ai_journal/rows.rs`). Le ré-ordonnancement par `rowid` départage les égalités de `updated_at`.
- [ ] **Step 4: Run** les trois fichiers de test et `cargo test -p mira-core --test storage_database` — Expected: PASS.
- [ ] **Step 5: Commit** `git add engine/crates/mira-core && git commit -m "feat(storage): migration v020 et dépôt des conversations IA"`

---

### Task 3: Contrat 1.4 — enregistrer, lister, lire une conversation

**Files:**
- Modify: `engine/crates/mira-core/src/api/contract/ai_requests.rs` (variantes `SaveAiConversation { id: String, title: String, payload_version: u32, payload: String }`, `ListAiConversations { limit: u32, before: Option<String> }`, `GetAiConversation { id: String }`), `ai_responses.rs` (variantes `Conversations(Vec<AiConversationMeta>)`, `Conversation(AiConversation)`, `ConversationMeta(AiConversationMeta)`), `risk/assistant.rs` (les trois en `Sensitive`), `api/handlers/ai/mod.rs` (routage)
- Create: `engine/crates/mira-core/src/api/handlers/ai/{conversation_input,save_conversation,list_conversations,get_conversation}.rs`
- Modify: `engine/crates/mira-vocabulary/src/contract/version.rs` (`minor: 4`)
- Modify (tests de contrat) : `tests/contract_examples/{ai_domain.rs,ai_samples.rs}` (un exemple par requête, un par réponse, `op_name` complété), `tests/contract_snapshot_ai.rs` (compte de 15 → 18 opérations, version `"1.4"`), tout test affirmant la version `1.3`/`minor: 3` (`grep -rn '"1.3"\|minor: 3' engine/crates`), régénérer `tests/snapshots/contract_v1.json` avec `MIRA_UPDATE_SNAPSHOT=1 cargo test -p mira-core --test contract_snapshot_requests_and_responses`
- Create (tests) : `tests/support_ai_conversations/mod.rs` (raccourcis `save`, `list`, `get` rendant `Result<_, String>` = code d'erreur, calqués sur `support_ai_api/calls.rs` ; moteur ouvert par `support_api::open` sur un dossier temporaire), `tests/api_ai_conversations.rs`, `tests/api_ai_conversations_limits.rs`

**Interfaces:**
- Consumes: T1, T2.
- Produces:
  - `conversation_input.rs` : `pub(super) fn validate_title(title: &str) -> Result<String, MiraError>` (rend le titre rogné, 1 à 80 caractères, sinon `invalid_input.ai_conversation_title`) ; `pub(super) fn validate_payload(payload: &str) -> Result<(), MiraError>` (≤ 2 097 152 octets sinon `invalid_input.ai_conversation_too_large` ; doit être un objet JSON sinon `invalid_input.ai_conversation_payload`) ; constantes `MAX_TITLE_CHARS = 80`, `MAX_PAYLOAD_BYTES = 2_097_152`, `MAX_CONVERSATIONS = 200`.
  - Handlers `pub(crate) fn handle_save_ai_conversation(engine: &Engine, id: &str, title: &str, payload_version: u32, payload: &str) -> Result<Response, MiraError>` (valide, puis dans une seule transaction `save` + `trim(MAX_CONVERSATIONS, id)` ; `now = engine.config.clock.now().as_millis()` ; rend `AiResponse::ConversationMeta`), `handle_list_ai_conversations(engine, limit, before)` (`limit` hors 1..=200 → `invalid_input.parameter`), `handle_get_ai_conversation(engine, id)`.
  - Formes JSON : `{"domain":"ai","request":{"op":"save_ai_conversation",…}}` ; réponses `{"type":"conversation_meta"|"conversations"|"conversation","data":…}`.

- [ ] **Step 1: Write the failing tests**
  - Contrat : mettre à jour `snapshot_file_holds_every_ai_example` (`"1.4"`) et le compte d'opérations (18) ; exemples ajoutés aux fichiers `ai_domain.rs`/`ai_samples.rs`.
  - `api_ai_conversations.rs` : `save_then_get_returns_same_payload` ; `save_twice_updates_in_place` (une seule ligne dans `list`) ; `list_is_newest_first_without_payload` (avancer l'horloge `ManualClock` entre deux `save`) ; `get_unknown_is_not_found`.
  - `api_ai_conversations_limits.rs` : `empty_or_blank_title_is_refused` (`""`, `"   "`) ; `title_counts_characters_not_bytes` (80 × `é` accepté, 81 refusé ; 80 × emoji `😀` accepté) ; `payload_over_2_mib_is_refused` (2 097 153 octets) et `payload_of_exactly_2_mib_is_accepted` ; `payload_must_be_a_json_object` (`"[]"`, `"42"`, `"pas du json"` → `invalid_input.ai_conversation_payload`) ; `limit_zero_or_over_200_is_refused` (`invalid_input.parameter`) ; `the_201st_conversation_evicts_the_oldest` (la 201e rend 200 lignes, la plus ancienne a disparu, la nouvelle est présente).
- [ ] **Step 2: Run** `cd engine && cargo test -p mira-core --test api_ai_conversations --test api_ai_conversations_limits --test contract_snapshot_ai` — Expected: FAIL.
- [ ] **Step 3: Implement** variantes, handlers, routage (`match` exhaustif), risque (`Sensitive`), version `1.4`, exemples ; régénérer l'instantané (commande ci-dessus) puis relire `git diff` du JSON : seules des lignes `ai.*` et `"contract": "1.4"` doivent changer.
- [ ] **Step 4: Run** les trois tests ci-dessus, puis `cargo test -p mira-core --test contract_snapshot_requests_and_responses --test contract_risk` — Expected: PASS (si `contract_risk` échoue sur l'ensemble des lectures, c'est que le risque d'une nouvelle requête n'est pas `Sensitive` : corriger le code, pas le test).
- [ ] **Step 5: Commit** `git add engine/crates && git commit -m "feat(api): contrat 1.4, enregistrer lister et lire les conversations IA"`

---

### Task 4: Contrat — renommer, supprimer, tout effacer

**Files:**
- Modify: `ai_requests.rs` (`RenameAiConversation { id: String, title: String }`, `DeleteAiConversation { id: String }`, `ClearAiConversations`), `risk/assistant.rs` (`Sensitive`), `handlers/ai/mod.rs`, `tests/contract_examples/{ai_domain.rs,ai_samples.rs}`, `tests/contract_snapshot_ai.rs` (18 → 21), `tests/snapshots/contract_v1.json` (régénérer)
- Create: `handlers/ai/{rename_conversation,delete_conversation,clear_conversations}.rs`
- Test: `tests/api_ai_conversations_changes.rs`

**Interfaces:**
- Consumes: T2 (`rename_ai_conversation`, `delete_ai_conversation`, `clear_ai_conversations`), `validate_title` de T3.
- Produces: `handle_rename_ai_conversation(engine, id: &str, title: &str) -> Result<Response, MiraError>` (rend `ConversationMeta`), `handle_delete_ai_conversation(engine, id: &str)` et `handle_clear_ai_conversations(engine)` (rendent `AiResponse::Done`).

- [ ] **Step 1: Write the failing tests** (`api_ai_conversations_changes.rs`) : `rename_changes_title_but_not_order` ; `rename_with_invalid_title_is_refused` (`invalid_input.ai_conversation_title`) ; `rename_unknown_is_not_found` ; `delete_removes_only_that_one` ; `delete_unknown_is_not_found` ; `clear_removes_everything_and_answers_done` ; `clear_leaves_ai_journal_untouched` (une ligne de journal existante reste).
- [ ] **Step 2: Run** `cd engine && cargo test -p mira-core --test api_ai_conversations_changes` — Expected: FAIL.
- [ ] **Step 3: Implement** variantes, handlers, routage, risque, exemples ; régénérer l'instantané.
- [ ] **Step 4: Run** `api_ai_conversations_changes`, `contract_snapshot_ai`, `contract_snapshot_requests_and_responses`, `contract_risk` — Expected: PASS.
- [ ] **Step 5: Commit** `git add engine/crates && git commit -m "feat(api): renommer supprimer et effacer les conversations IA"`

---

### Task 5: Contrat — interrupteur de l'historique et absence de contenu

**Files:**
- Modify: `ai_requests.rs` (`SetAiHistoryEnabled { enabled: bool }`, `GetAiHistoryEnabled`), `ai_responses.rs` (`HistoryEnabled(bool)`), `risk/assistant.rs` (`SetAiHistoryEnabled` → `Sensitive`, `GetAiHistoryEnabled` → `Read`), `handlers/ai/mod.rs`, `handlers/ai/save_conversation.rs` (refus si désactivé), exemples et `contract_snapshot_ai.rs` (21 → 23), `tests/contract_risk.rs` (ajouter `"ai.get_ai_history_enabled"` à `READS`, avant `"ai.list_ai_connections"`), instantané régénéré
- Create: `handlers/ai/{set_history_enabled,get_history_enabled}.rs`
- Test: `tests/api_ai_history_setting.rs`, `tests/api_ai_conversations_privacy.rs`

**Interfaces:**
- Consumes: `ai_history_enabled` / `set_ai_history_enabled` (T2).
- Produces: `handle_set_ai_history_enabled(engine, enabled: bool)` et `handle_get_ai_history_enabled(engine)`, tous deux rendant `AiResponse::HistoryEnabled(bool)`. `handle_save_ai_conversation` lit le réglage dans la même transaction que l'écriture et rend `incompatible_state.ai_history_disabled` s'il est coupé (avant toute autre écriture).

- [ ] **Step 1: Write the failing tests**
  - `api_ai_history_setting.rs` : `enabled_by_default` ; `disabling_refuses_save` (`incompatible_state.ai_history_disabled`) ; `disabling_keeps_existing_conversations_readable_and_deletable` ; `re_enabling_allows_save_again` ; `setting_survives_reopening_the_profile`.
  - `api_ai_conversations_privacy.rs` : après un `save` dont le payload contient la chaîne `SECRET-PAYLOAD-123` et le titre `SECRET-TITLE-456` : `ai_journal` vide de ces chaînes (`ListAiJournal` sérialisé) ; `events_text(&engine)` (support `leaks.rs`) n'en contient pas ; l'erreur d'un `save` refusé (titre invalide, payload trop gros) ne contient pas le payload ; `format!("{:?}", erreur)` non plus.
- [ ] **Step 2: Run** `cd engine && cargo test -p mira-core --test api_ai_history_setting --test api_ai_conversations_privacy` — Expected: FAIL.
- [ ] **Step 3: Implement** variantes, handlers, refus dans `save`, risque, exemples ; régénérer l'instantané.
- [ ] **Step 4: Run** les deux tests, puis `contract_snapshot_ai`, `contract_snapshot_requests_and_responses`, `contract_risk`, `api_ai_conversations`, `api_ai_conversations_limits` — Expected: PASS.
- [ ] **Step 5: Commit** `git add engine/crates && git commit -m "feat(api): interrupteur de l'historique des conversations IA"`

---

### Task 6: App — sérialisation du fil et `AiHistoryAdmin`

**Files:**
- Create: `app/lib/engine/ai_history_models.dart` (`AiConversationMeta`, `AiConversationDoc`), `app/lib/engine/ai_history_admin.dart`, `app/lib/state/chat_message_json.dart` (sérialisation de `ChatMsg`)
- Modify: `app/test/fake_ai_engine.dart` (nouvelles opérations, voir Step 3)
- Test: `app/test/chat_message_json_test.dart`, `app/test/ai_history_admin_test.dart`

**Interfaces:**
- Produces:
  - `class AiConversationMeta { final String id; final String title; final DateTime createdAt; final DateTime updatedAt; final int payloadVersion; factory AiConversationMeta.fromJson(Map<String, dynamic>) }` — dates lues comme `AiJournalRow.fromJson` lit `at` (millisecondes).
  - `class AiConversationDoc { final AiConversationMeta meta; final String payload; }`
  - `const chatPayloadVersion = 1;`
  - `String encodeChat(List<ChatMsg> messages)` — texte JSON `{"messages":[…]}` ; chaque message : `role`, `text`, `draft`, `refs`, `sources`, `tools` (liste de `{id, name, summary, state, results, denied_kind}`, `state` = `ToolState.name`), `failed`, `note`, `masked`, `engine`. `shown` et `done` ne sont pas écrits.
  - `List<ChatMsg>? decodeChat(String payload, int version)` — `null` si la version ≠ 1, le JSON est invalide ou la forme est mauvaise (jamais d'exception) ; les messages rendus ont `done: true` et `shown == text`.
  - `class AiHistoryAdmin { const AiHistoryAdmin(this.client); Future<List<AiConversationMeta>> list({int limit = 50, String? before}); Future<AiConversationDoc> get(String id); Future<AiConversationMeta> save({required String id, required String title, required int payloadVersion, required String payload}); Future<AiConversationMeta> rename(String id, String title); Future<void> delete(String id); Future<void> clearAll(); Future<bool> enabled(); Future<bool> setEnabled(bool enabled); }` — opérations `list_ai_conversations`, `get_ai_conversation`, `save_ai_conversation`, `rename_ai_conversation`, `delete_ai_conversation`, `clear_ai_conversations`, `get_ai_history_enabled`, `set_ai_history_enabled` ; lecture des réponses comme `AiAdmin._send` (`r['data']['data']`).

- [ ] **Step 1: Write the failing tests**
  - `chat_message_json_test.dart` : `round_trip_keeps_every_field` (message avec `draft`, `refs`, deux `sources`, deux outils dont un `denied` avec `deniedKind`, `failed`, `note`, `maskedCount`, `fromEngine`) ; `decoded_messages_are_done_and_shown` ; `unknown_version_gives_null` ; `corrupted_json_gives_null` ; `wrong_shape_gives_null` (`{"messages": 3}`, `[]`).
  - `ai_history_admin_test.dart` (avec `FakeAiEngine`) : `save_sends_the_expected_request` (op, `id`, `title`, `payload_version`, `payload`) ; `list_parses_meta_and_forwards_before` ; `get_returns_payload_text` ; `enabled_and_set_enabled_round_trip` ; `engine_error_becomes_EngineException` (`failOp`).
- [ ] **Step 2: Run** `cd app && flutter test test/chat_message_json_test.dart test/ai_history_admin_test.dart` — Expected: FAIL (fichiers absents).
- [ ] **Step 3: Implement** modèles, admin, sérialisation, et dans `FakeAiEngine` : champs `final conversations = <String, Map<String, dynamic>>{}` (clé = `id`, valeur = méta + `payload`) et `bool historyEnabled = true`, branches de `_answer` pour les huit opérations (réponses `conversations`, `conversation`, `conversation_meta`, `history_enabled`, `done` ; `save` refusé par `EngineException('incompatible_state.ai_history_disabled', '')` si `!historyEnabled` ; `updated_at` croissant ; liste triée du plus récent au plus ancien, `before` comme `_page`).
- [ ] **Step 4: Run** les deux tests — Expected: PASS ; puis `flutter test test/ai_admin_test.dart` — Expected: PASS (le faux moteur reste compatible).
- [ ] **Step 5: Commit** `git add app/lib/engine/ai_history_models.dart app/lib/engine/ai_history_admin.dart app/lib/state/chat_message_json.dart app/test && git commit -m "feat(app): sérialisation du fil et client de l'historique IA"`

---

### Task 7: App — `ChatHistoryState`

**Files:**
- Create: `app/lib/state/chat_history_state.dart`
- Test: `app/test/chat_history_state_test.dart`

**Interfaces:**
- Consumes: `AiHistoryAdmin`, `AiConversationMeta` (T6) ; modèle de comportement : `app/lib/state/ai_journal_state.dart` (pages, génération, `_disposed`).
- Produces: `const chatHistoryPage = 50;` et
  `class ChatHistoryState extends ChangeNotifier { ChatHistoryState(this.admin); final AiHistoryAdmin? Function() admin; List<AiConversationMeta> items; bool loaded; bool loading; bool hasMore; bool enabled; String? error; bool get available; Future<void> load(); Future<void> more(); Future<void> rename(String id, String title); Future<void> delete(String id); Future<void> clearAll(); Future<void> setEnabled(bool value); }`
  - `available` = `admin() != null` (faux en démo). `load()` relit l'interrupteur (`enabled`) puis la première page (`chatHistoryPage + 1` demandées pour calculer `hasMore`). `more()` demande après `items.last.id`. Une lecture périmée (génération) est ignorée. Chaque échec devient `error` via `engineErrorText`, sans exception. `delete`, `rename`, `clearAll` relisent la liste. `setEnabled` met `enabled` à la valeur confirmée par le moteur ; en cas d'échec `enabled` ne change pas et `error` est posé.

- [ ] **Step 1: Write the failing tests** : `load_reads_enabled_and_first_page` ; `has_more_when_a_51st_row_exists` ; `more_appends_after_the_last_id` ; `stale_load_is_ignored` (deux `load` croisés, le premier terminé en dernier n'écrase pas) ; `rename_updates_title_in_list` ; `delete_removes_row` ; `clear_all_empties_list` ; `engine_error_is_translated_and_not_thrown` ; `set_enabled_false_keeps_existing_items` ; `set_enabled_failure_keeps_previous_value` ; `unavailable_without_admin` (`available == false`, `load()` ne lève pas).
- [ ] **Step 2: Run** `cd app && flutter test test/chat_history_state_test.dart` — Expected: FAIL.
- [ ] **Step 3: Implement** `ChatHistoryState` en suivant `AiJournalState`.
- [ ] **Step 4: Run** le test — Expected: PASS.
- [ ] **Step 5: Commit** `git add app/lib/state/chat_history_state.dart app/test/chat_history_state_test.dart && git commit -m "feat(app): état de la liste des conversations IA"`

---

### Task 8: App — enregistrement automatique, ouverture et nouvelle conversation

**Files:**
- Modify: `app/lib/state/chat_controller.dart`, `app/lib/state/store.dart` (construction du contrôleur, `historyAdmin`, `history`, `clearChat`), `app/lib/state/chat_message.dart` (aucun changement attendu)
- Create: `app/lib/state/chat_conversation.dart` (titre et identifiant), `app/test/chat_history_save_test.dart`, `app/test/chat_history_open_test.dart`
- Modify: `app/lib/i18n/lang/en.dart`

**Interfaces:**
- Consumes: T6 (`encodeChat`, `decodeChat`, `AiHistoryAdmin`, `chatPayloadVersion`), T7 (`ChatHistoryState`).
- Produces:
  - `String conversationTitle(String firstQuestion)` (`chat_conversation.dart`) : retours à la ligne et espaces multiples ramenés à un espace, rogné, coupé à **80 caractères** (`String.characters`, pas `substring`), jamais vide (repli `t('Nouvelle conversation')`). `String newConversationId()` : UUID v4 sans nouvelle dépendance si `uuid` n'est pas déjà dans `pubspec.yaml` (sinon générer 16 octets aléatoires de `Random.secure()` formatés en hexadécimal).
  - `ChatController` : nouveau paramètre nommé `required this.history` (`ChatHistoryState Function()`), `String? conversationId`, `bool saveFailed`, `Future<void> open(String id)`, `void startNew()`, `Future<void> _save()` appelée par `_finish`. `clear()` devient `startNew()` (nouvel identifiant `null`, fil `[welcome()]`) ; `reset()` remet aussi `conversationId = null`.
  - `MailStore` : `AiHistoryAdmin? get historyAdmin` (construit sur `aiAdmin!.client`, `null` si pas d'`aiAdmin`), `late final ChatHistoryState history = ChatHistoryState(() => historyAdmin)` ; `clearChat()` appelle `chatCtl.startNew()`.
  - Règles de `_save()` : ne fait rien si `demo`, si `!history().enabled`, si `chat.length <= 1` (accueil seul), si la dernière réponse n'est pas `done`, ou s'il n'y a pas d'`admin`. Crée `conversationId` à la première sauvegarde ; titre = `conversationTitle(` première question `)` ; payload = `encodeChat(chat.skip(1))` (accueil exclu) ; un échec pose `saveFailed = true` et notifie, sans lever ; un succès remet `saveFailed = false` et recharge `history().load()`.
  - `Future<String?> open(String id)` rend `null` en cas de succès, sinon le texte d'erreur à montrer : `null` aussi (sans rien faire) si `busy` ; `admin.get(id)` qui échoue → `engineErrorText(e)` ; `decodeChat` qui rend `null` → `t('Conversation illisible')`. Dans les deux cas d'erreur le fil courant et `conversationId` ne changent pas. En succès : `chat = [welcome(), ...messages]`, `conversationId = id`, `notifyListeners()`.

- [ ] **Step 1: Write the failing tests**
  - `chat_history_save_test.dart` (harnais `ai_panel_harness.dart` + `withDefaultConnection()`, scripts de flux de `FakeAiEngine`) : `one_save_per_finished_turn` (deux tours → deux `save_ai_conversation`, même `id`) ; `no_save_while_generating` (`manual = true`) ; `no_save_in_demo` ; `no_save_when_history_disabled` ; `no_save_when_only_welcome` ; `disabling_mid_run_skips_that_turn` (interrupteur coupé via `history.setEnabled(false)` avant la fin du flux) ; `title_is_first_question_cleaned_and_truncated` (question de 120 caractères avec retour à la ligne ; question d'emoji ≤ 80 caractères intacte) ; `welcome_is_not_in_payload` ; `save_failure_does_not_break_chat_and_retries` (`failOp` sur `save_ai_conversation` au premier tour : `saveFailed == true`, la réponse reste affichée ; au tour suivant, `save` repart et `saveFailed == false`).
  - `chat_history_open_test.dart` : `open_replaces_thread_and_keeps_id` ; `next_turn_after_open_saves_into_same_conversation` ; `open_is_ignored_while_busy` ; `unreadable_payload_keeps_current_thread_and_returns_error_text` (version 9 : `open` rend `Conversation illisible`, fil et `conversationId` inchangés) ; `engine_error_on_get_returns_translated_text` ; `start_new_gives_fresh_id_and_keeps_old_saved` ; `reset_forgets_conversation_id`.
- [ ] **Step 2: Run** `cd app && flutter test test/chat_history_save_test.dart test/chat_history_open_test.dart` — Expected: FAIL.
- [ ] **Step 3: Implement** ; mettre à jour tous les sites qui construisent `ChatController` (`store.dart:167` et les tests existants : `grep -rn "ChatController(" app/test app/lib`). Aucun commentaire dans les corps, `///` en français avant chaque membre.
- [ ] **Step 4: Run** les deux nouveaux tests puis `flutter test test/chat_controller_test.dart test/chat_controller_edge_test.dart test/chat_controller_pending_test.dart test/chat_privacy_test.dart test/assistant_panel_ai_test.dart` — Expected: PASS (comportements existants inchangés).
- [ ] **Step 5: Commit** `git add app/lib app/test && git commit -m "feat(app): enregistrement automatique et reprise des conversations IA"` (vérifier `git status` : `ai_connection_card.dart` n'est pas indexé).

---

### Task 9: App — vue « Historique » dans le panneau assistant

**Files:**
- Create: `app/lib/ui/chat_history_view.dart`, `app/lib/ui/chat_history_dialogs.dart` (renommer, supprimer, tout effacer), `app/test/chat_history_view_test.dart`
- Modify: `app/lib/ui/assistant_panel.dart` (bouton d'en-tête, bascule `showHistory`, bandeau « historique désactivé », masquage du contexte, du fil et de la saisie), `app/lib/i18n/lang/en.dart`, `app/lib/i18n/engine_errors.dart` (cinq codes, voir T10 Step 3)

**Interfaces:**
- Consumes: `ChatHistoryState` (T7), `ChatController.open/startNew/conversationId/saveFailed` (T8), icône `'clock'` (`theme/icons.dart`).
- Produces: `class ChatHistoryView extends StatelessWidget { const ChatHistoryView({super.key, required this.state, required this.currentId, required this.busy, required this.onOpen, required this.onNewAfterClear}); }` où `onOpen` est `Future<String?> Function(String id)` (le texte rendu par `ChatController.open` s'affiche dans le toast existant `ToastHost`, et la vue reste affichée en cas d'erreur), avec `ValueKey` : `chat-history-open` (bouton d'en-tête), `chat-history-row-<id>`, `chat-history-menu-<id>`, `chat-history-rename`, `chat-history-delete`, `chat-history-clear`, `chat-history-more`, `chat-history-empty`, `chat-history-disabled-note`, `chat-history-unavailable`.
  - Lignes de 48 px minimum : titre (une ligne, ellipse), date relative (`data/format.dart`), ligne de la conversation ouverte surlignée, menu toujours visible (Renommer, Supprimer). Clic = `onOpen(id)` puis retour au fil ; grisé avec info-bulle `t('Attends la fin de la réponse')` si `busy`. Pied : « Afficher plus » si `hasMore`, « Tout effacer » (confirmation). États vides : `t('Aucune conversation enregistrée')` ou, si `!state.enabled`, `t('L’historique est désactivé')` avec lien vers les réglages. Bandeau du fil quand `!enabled` : `t('Historique désactivé : cette conversation ne sera pas enregistrée')`. Avertissement discret quand `saveFailed` : `t('Conversation non enregistrée : nouvel essai au prochain message')`.
  - « Tout effacer » confirmé → `state.clearAll()` puis `onNewAfterClear()` (le panneau appelle `chatCtl.startNew()`). Supprimer la conversation ouverte → `startNew()`.
  - Le bouton d'en-tête est absent en démo (`state.available == false`) ; en démo la vue n'est jamais atteignable (`chat-history-unavailable` n'apparaît que si la vue est forcée dans un test).

- [ ] **Step 1: Write the failing tests** (`chat_history_view_test.dart`, harnais `ai_panel_harness.dart`, écrans 1500 px et 400 px) : `header_button_hidden_in_demo` ; `rows_show_title_and_highlight_current` ; `tap_row_opens_and_returns_to_thread` ; `rows_disabled_while_busy` ; `rename_dialog_limits_to_80_chars_and_updates_list` ; `delete_asks_confirmation_and_removes_row` ; `deleting_the_open_conversation_starts_a_fresh_thread` ; `clear_all_asks_confirmation_and_resets_open_thread` (le prochain message crée un nouvel `id`, aucune ressuscitation) ; `empty_state_text` ; `disabled_state_text_and_note_in_thread` ; `unreadable_row_can_be_deleted` (payload version 9 : ligne présente, suppression OK, ouverture affiche `Conversation illisible`) ; `narrow_width_keeps_actions_visible` (400 px, pas de débordement : aucun `RenderFlex overflowed`).
- [ ] **Step 2: Run** `cd app && flutter test test/chat_history_view_test.dart` — Expected: FAIL.
- [ ] **Step 3: Implement** vue, dialogues (modèle : `_ClearDialog` de `ai_journal_view.dart`), bascule du panneau ; n'utiliser que `Btn`, `IBtn`, `Mono`, `RuleChip`, `DashedBox`, `Fonts`, `context.pal`. Le panneau charge la liste à l'ouverture de la vue (`state.load()`).
- [ ] **Step 4: Run** `flutter test test/chat_history_view_test.dart test/assistant_panel_ai_test.dart test/assistant_suggestions_test.dart test/chat_mentions_panel_test.dart` — Expected: PASS.
- [ ] **Step 5: Commit** `git add app/lib app/test && git commit -m "feat(app): vue Historique du panneau assistant"`

---

### Task 10: App — réglages (interrupteur, effacement) et traductions

**Files:**
- Create: `app/lib/ui/ai_history_section.dart`, `app/test/ai_history_section_test.dart`
- Modify: `app/lib/ui/ai_settings.dart` (insérer la section), `app/lib/i18n/engine_errors.dart`, `app/lib/i18n/lang/en.dart`

**Interfaces:**
- Consumes: `ChatHistoryState` (T7) ; l'état partagé est celui de `MailStore.history` (T8), passé par `AiSettings` via `context.read<MailStore>().history` (ou un paramètre `history` si `AiSettings` est utilisé hors `Provider` dans les harnais : suivre ce que fait déjà `ai_settings_harness.dart`).
- Produces: `class AiHistorySection extends StatelessWidget { const AiHistorySection({super.key, required this.state, required this.onCleared}); }` — `ValueKey` `ai-history-toggle` (interrupteur), `ai-history-clear` (bouton), `ai-history-note`. Texte : `t('Historique des conversations')`, `t('Conserver l’historique des conversations')`, `t('Enregistre sur cet appareil le texte affiché des conversations. Les contenus envoyés au modèle en coulisses ne sont pas conservés.')`, `t('Effacer tout l’historique')` (confirmation). Interrupteur : `RuleSegment` (oui/non) comme `ai-view-toggle`.
- Codes à ajouter à `engineErrorTexts` (le test existant « aucun code ne manque » l'exige) : `incompatible_state.ai_history_disabled` « L’historique des conversations est désactivé », `invalid_input.ai_conversation_title` « Le titre de la conversation est vide ou trop long (80 caractères au plus) », `invalid_input.ai_conversation_too_large` « Cette conversation est trop volumineuse pour être enregistrée », `invalid_input.ai_conversation_payload` « Conversation illisible », `not_found.ai_conversation` « Cette conversation n’existe plus » — chacun avec sa traduction anglaise dans `en.dart`.

- [ ] **Step 1: Write the failing tests** : `toggle_is_on_by_default` ; `toggling_off_calls_set_ai_history_enabled_false_and_keeps_items` ; `clear_button_asks_confirmation_then_clears` ; `clear_calls_onCleared` ; `section_shows_note_text` ; test existant des codes d'erreur (`grep -rln "engineErrorTexts" app/test`) passe avec les cinq nouveaux codes ; test existant de couverture i18n (`grep -rln "en.dart\|i18n" app/test`) passe.
- [ ] **Step 2: Run** `cd app && flutter test test/ai_history_section_test.dart` — Expected: FAIL.
- [ ] **Step 3: Implement** la section, l'insérer dans `AiSettings` au-dessus du sélecteur Connexions/Journal, ajouter les entrées `engine_errors.dart` et `en.dart`.
- [ ] **Step 4: Run** `flutter test test/ai_history_section_test.dart test/ai_settings_test.dart test/ai_journal_view_test.dart` puis les tests i18n/erreurs repérés au Step 1 — Expected: PASS.
- [ ] **Step 5: Commit** `git add app/lib app/test && git commit -m "feat(app): réglage de l'historique des conversations IA"`

---

### Task 11: Documentation et vérification finale

**Files:**
- Modify: `docs/ARCHITECTURE-IMPLEMENTEE.md` (nouvelle sous-section `### 6.9 Historique des conversations` après le dernier §6.x ; mettre à jour la phrase « Version du contrat `1.3` » en `1.4` avec « historique des conversations en 1.4 », et la mention de la dernière migration), `docs/superpowers/specs/2026-11-01-ai-conversation-history-design.md` (statut « implémenté »)

**Interfaces:** aucune.

- [ ] **Step 1: Écrire §6.9** : table `ai_conversations` (v020), huit requêtes et réponses, risques (`Sensitive`/`Read`), bornes (80 caractères, 2 Mio, 200), clé `ai_history_enabled`, absence de contenu dans journal/événements/erreurs, contenu enregistré (texte affiché, accueil exclu, sans contexte ni `tool_result`), limites (pas de recherche dans le contenu, pas de titre généré, pas d'historique en démo).
- [ ] **Step 2: Vérifier le moteur** — `cd engine && cargo fmt --check && cargo clippy --all-targets --all-features -- -D warnings && cargo test -p mira-vocabulary -p mira-core` — Expected: aucun avertissement, tous verts. Puis `tools/check-norms` (seul, sans `./ci.sh`) — Expected: vert ; corriger toute fonction > 30 lignes, fichier > 6 fonctions ou commentaire interne signalé.
- [ ] **Step 3: Vérifier l'app** — `cd app && dart format --set-exit-if-changed lib test && flutter analyze && flutter test` — Expected: aucun problème d'analyse, tous les tests verts.
- [ ] **Step 4: Vérifier le périmètre** — `git status --short` : seul `app/lib/ui/ai_connection_card.dart` reste modifié et non indexé ; `git diff dev --stat -- bridge app/lib/src/rust` : vide (pont inchangé).
- [ ] **Step 5: Commit** `git add docs && git commit -m "docs: historique des conversations IA (contrat 1.4)"`
- [ ] **Step 6: Demander à l'utilisateur** s'il veut lancer `./ci.sh` (ne pas le lancer sans accord).
