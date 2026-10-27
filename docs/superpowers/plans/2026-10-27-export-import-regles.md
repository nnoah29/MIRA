# Export et import des règles en YAML — plan d'implémentation

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Exporter toutes les règles dans un fichier YAML et les réimporter (y compris sur une autre machine), avec rapport par règle, doublons ignorés ou remplacés, et références portables.

**Architecture:** Le moteur convertit du texte YAML (aucune I/O fichier). Une règle est sérialisée en JSON (`serde_json::Value`), puis un parcours générique remplace les identifiants locaux (`account`, `accounts`, `mailbox`, `label`, `template`) par des références en clair, et inversement à l'import. Deux opérations s'ajoutent au domaine `Rule` (`ExportRules`, `ImportRules`) ; Flutter lit et écrit le fichier avec `file_picker` et affiche le rapport.

**Tech Stack:** Rust 2024 (workspace `engine/`, toolchain 1.99.0), `yaml_serde` (repli `serde_norway`), rusqlite ; Flutter/Dart (`app/`), `cargo test` / `flutter test`.

**Spec:** `docs/superpowers/specs/2026-10-27-export-import-regles-design.md`

## Global Constraints

- Branche `feat/yaml-import-export` ; commits à l'identité de l'utilisateur, **sans trailer d'attribution**.
- Pas de `./ci.sh` sans l'accord de l'utilisateur : vérifier avec `cargo test -p <crate> --test <fichier>`, `cargo clippy`, `flutter test <fichier>`, puis demander.
- Normes (`tools/check-norms`) : fonction ≤ 30 lignes, fichier ≤ 6 fonctions avec corps (tests compris, tests en fichiers séparés), aucun commentaire dans un corps de fonction, une ligne `///` en français avant chaque `fn`, `//!` de 1 à 3 lignes en tête de fichier, identifiants en anglais. Pas de `unsafe`, pas de `unwrap`/`expect`/`panic!` hors tests. Le cœur ne nomme aucun provider.
- Enveloppe : `mira_export: { kind: rules, version: 1 }` vérifiée avant tout ; `kind` ≠ `rules` ou version inconnue refuse le fichier entier.
- Références : compte → adresse ; boîte → adresse du compte + nom serveur de la boîte ; libellé et modèle → nom (casse et espaces de bord ignorés).
- Une référence non résolue refuse **la règle**, pas le fichier. Doublon (même `id`) : ignoré, ou remplacé si `replace_existing`.
- Bornes : fichier ≤ 2 Mo (2 097 152 octets), ≤ 500 règles → `invalid_input.export_too_large`.
- Erreurs de fichier entier : `invalid_input.yaml_syntax` (ligne et colonne, aucun extrait du contenu), `invalid_input.export_version_unsupported`, `invalid_input.export_kind_mismatch`, `invalid_input.export_too_large`.
- Aucun contenu de règle (noms, motifs, adresses) dans journaux, erreurs ni événements.
- Événement existant : `EventKind::RuleChanged { rule }` (il n'y a pas de `RulesChanged`) ; l'import l'émet une fois par règle écrite, jamais en lecture, rejeu ou import sans effet.
- Décisions à consigner dans `docs/DECISIONS-AGENT.md` (numéro suivant la séquence) : crate YAML retenue et repli ; parcours générique par noms de champs ; bornes ; fichier d'import conserve les `id` ; migration v014.

## Review Focus

1. Anchors/alias YAML en cascade (« bombe ») sous 2 Mo : refusé sans blocage ni explosion mémoire.
2. Fichier vide, ou dont la racine n'est pas un objet : `invalid_input.yaml_syntax` ou `export_kind_mismatch`, jamais une panique.
3. Fichier enregistré avec BOM UTF-8 et fins de ligne CRLF (édition sous Windows) : accepté.
4. Règle sans `id`, ou `id` vide : refusée `invalid_rule`, jamais importée sous un identifiant vide.
5. Même clé d'idempotence avec `replace_existing` différent : `idempotency_conflict`.

---

## File Structure

| Fichier | Responsabilité |
|---|---|
| `mira-core/src/portability/{mod,envelope}.rs` (créer) | enveloppe `mira_export`, version, `kind`, bornes de taille, lecture/écriture YAML |
| `mira-core/src/portability/rules/{mod,refs,to_portable,from_portable}.rs` (créer) | trait `RuleRefs`, parcours générique d'une règle dans les deux sens |
| `mira-core/src/portability/rules/tx_refs.rs` (créer) | `RuleRefs` implémenté sur `Tx` |
| `mira-vocabulary/src/domain/rule/import_report.rs` (créer) | `ImportReport`, `ImportEntry`, `ImportOutcome`, `ImportRefusal` |
| `mira-core/src/api/handlers/rule/{export,import,import_one}.rs` (créer) | un handler par opération |
| `mira-core/src/storage/migrations/v014_rule_idempotency_result.sql` (créer) | colonne `result` de `rule_idempotency` |
| `app/lib/engine/rule_admin.dart`, `app/lib/ui/rules_settings.dart`, `app/lib/ui/rules_import_dialog.dart` (créer ce dernier) | côté Flutter |

---

### Task 1: Format portable (fonctions pures)

**Files:**
- Create: `mira-core/src/portability/{mod,envelope}.rs`, `mira-core/src/portability/rules/{mod,refs,to_portable,from_portable}.rs`
- Modify: `engine/Cargo.toml` (dépendance de workspace `yaml_serde`), `mira-core/Cargo.toml`, `mira-core/src/lib.rs`, `mira-vocabulary/src/error/codes/invalid_input.rs`
- Test: `mira-core/tests/portability_envelope.rs`, `mira-core/tests/portability_rules_format.rs`

**Interfaces:**
- Produces :
  - `portability::envelope::wrap(kind: &str, items_key: &str, items: Vec<serde_json::Value>) -> Result<String, MiraError>` : produit le YAML avec `mira_export: {kind, version: 1}` puis la liste sous `items_key`.
  - `portability::envelope::unwrap(text: &str, kind: &str, items_key: &str) -> Result<Vec<serde_json::Value>, MiraError>` : contrôle taille (≤ 2 097 152 octets), BOM/CRLF acceptés, syntaxe, `kind`, version, nombre d'éléments (≤ 500), puis rend les éléments.
  - `trait RuleRefs { fn account_address(&self, id: &AccountId) -> Option<String>; fn account_by_address(&self, address: &str) -> Option<AccountId>; fn mailbox_ref(&self, id: &MailboxId) -> Option<(String, String)>; fn mailbox_by_ref(&self, address: &str, name: &str) -> Option<MailboxId>; fn label_name(&self, id: &LocalLabelId) -> Option<String>; fn label_by_name(&self, name: &str) -> Option<LocalLabelId>; fn template_name(&self, id: &TemplateId) -> Option<String>; fn template_by_name(&self, name: &str) -> Option<TemplateId>; }`
  - `portability::rules::to_portable(rule: &Rule, refs: &dyn RuleRefs) -> Result<serde_json::Value, MiraError>` : JSON de la règle, références remplacées par `{ account: <adresse> }`-style en clair (voir ci-dessous).
  - `portability::rules::from_portable(value: serde_json::Value, refs: &dyn RuleRefs) -> Result<Rule, PortableError>` avec `enum PortableError { UnknownAccount, UnknownMailbox, UnknownLabel, UnknownTemplate, Invalid }` ; `Invalid` couvre champ inconnu, `id` absent ou vide, forme incorrecte.
- Forme portable (le parcours remplace la valeur de la clé, sans redescendre dedans) : clé `account` → `"adresse"` ; clé `accounts` (tableau) → tableau d'adresses ; clé `mailbox` → `{ account: "adresse", name: "nom serveur" }` ; clé `label` → `"nom"` ; clé `template` → `"nom"`.
- Erreurs ajoutées : `YAML_SYNTAX`, `EXPORT_VERSION_UNSUPPORTED`, `EXPORT_KIND_MISMATCH`, `EXPORT_TOO_LARGE` dans `invalid_input.rs`.

- [ ] **Step 1: Écrire `portability_envelope.rs`** (appels directs à `envelope::wrap` / `unwrap`) :
  - `wrap_then_unwrap_returns_items_in_order`
  - `unknown_version_refuses_whole_file` → `export_version_unsupported`
  - `other_kind_refuses_whole_file` → `export_kind_mismatch`
  - `syntax_error_reports_line_and_column_without_content` : texte `"mira_export: [\nSECRET-MARK-42"` ; le message contient « ligne » et « colonne » et **pas** `SECRET-MARK-42`.
  - `empty_or_non_object_root_is_refused_without_panic` (Review Focus 2)
  - `bom_and_crlf_are_accepted` (Review Focus 3)
  - `more_than_500_items_is_too_large` ; `text_over_2_mib_is_too_large`
  - `alias_bomb_is_refused_without_blowup` (Review Focus 1) : alias imbriqués sur ~100 Ko ; le test doit terminer et rendre une erreur typée.

- [ ] **Step 2: Lancer pour voir l'échec** — `cd engine && cargo test -p mira-core --test portability_envelope` ; attendu : erreur de compilation (module absent).

- [ ] **Step 3: Implémenter `envelope`** — ajouter `yaml_serde` au workspace (vérifier d'abord `cargo build` avec la toolchain 1.99.0 ; sinon basculer sur `serde_norway` et le consigner). Lire le texte en `serde_json::Value` via la crate YAML ; position d'erreur extraite de l'erreur de la crate ; contrôle de taille **avant** analyse. Si la crate n'offre pas de limite d'expansion des alias, les refuser explicitement (rejet de tout alias/anchor, consigné).

- [ ] **Step 4: Relancer** — 9 tests PASS.

- [ ] **Step 5: Écrire `portability_rules_format.rs`** avec un `FakeRefs` (table en mémoire implémentant `RuleRefs`) :
  - `round_trip_keeps_rule_identical` : règle avec `scope` comptes, `MoveTo`, `AddLocalLabel`, `ReplyWithTemplate`, conditions `InMailbox`, `FromAccount`, `HasLocalLabel` → `from_portable(to_portable(r)) == r`.
  - `portable_form_uses_names_not_ids` : le JSON portable ne contient aucun identifiant local ; contient l'adresse, le chemin et les noms.
  - `unknown_references_map_to_matching_errors` : un test par `UnknownAccount`, `UnknownMailbox`, `UnknownLabel`, `UnknownTemplate`, y compris dans une condition et dans `scope.accounts`.
  - `name_lookup_ignores_case_and_edge_spaces` (via `FakeRefs`, qui applique la même normalisation que `name_key`).
  - `unknown_field_is_invalid` : `{"…","nom_inconnu": 1}` → `Invalid`.
  - `missing_or_empty_id_is_invalid` (Review Focus 4).

- [ ] **Step 6: Implémenter `refs`, `to_portable`, `from_portable`** — un parcours récursif sur `serde_json::Value` aiguillé par le nom de clé (`match` exhaustif sur les cinq noms). Le contrôle de champ inconnu : après résolution, désérialiser en `Rule`, resérialiser en `Value` et comparer à l'entrée ; toute différence → `Invalid` (le fichier exporté contient toujours `schema_version`, donc pas de faux positif ; un fichier sans `schema_version` est donc `Invalid`).

- [ ] **Step 7: Relancer** — `cargo test -p mira-core --test portability_envelope --test portability_rules_format` ; `cargo clippy -p mira-core -p mira-vocabulary --all-targets -- -D warnings` ; `tools/check-norms` ; attendu : vert.

- [ ] **Step 8: Commit**

```bash
git add engine docs
git commit -m "feat(règles): enveloppe d'export YAML et conversion portable des règles"
```

---

### Task 2: Contrat, résolution sur la base et export

**Files:**
- Create: `mira-vocabulary/src/domain/rule/import_report.rs`, `mira-core/src/portability/rules/tx_refs.rs`, `mira-core/src/api/handlers/rule/export.rs`
- Modify: `mira-vocabulary/src/domain/rule/mod.rs`, `mira-vocabulary/src/lib.rs`, `mira-core/src/api/contract/{rule_requests,rule_responses}.rs`, `mira-core/src/api/handlers/rule/mod.rs`, `mira-core/src/portability/rules/mod.rs`
- Test: `mira-core/tests/api_rules_export.rs`, `mira-core/tests/contract_examples/rule_domain.rs` (+ `snapshot.rs` si nécessaire), `mira-core/tests/contract_snapshot_rules.rs`

**Interfaces:**
- Consumes: Task 1 (`RuleRefs`, `to_portable`, `envelope::wrap`).
- Produces :
  - `RuleRequest::ExportRules` ; `RuleResponse::RulesExport { yaml: String }` ; `RuleRequest::ImportRules { yaml: String, replace_existing: bool, idempotency_key: IdempotencyKey }` ; `RuleResponse::RulesImport(ImportReport)` (la variante d'import est déclarée ici, branchée à la Task 3).
  - `ImportReport { entries: Vec<ImportEntry> }`, `ImportEntry { position: u32, name: String, outcome: ImportOutcome }`, `enum ImportOutcome { Imported, Replaced, AlreadyPresent, Refused { reason: ImportRefusal } }`, `enum ImportRefusal { UnknownAccount, UnknownMailbox, UnknownLabel, UnknownTemplate, InvalidRule { code: String } }` (`position` commence à 1). Tous `Debug, Clone, PartialEq, Eq, Serialize, Deserialize`, `rename_all = "snake_case"`.
  - `impl RuleRefs for Tx<'_>` : comptes via `list_accounts` (adresse), boîtes via `find_mailbox_by_name`/`get_mailbox`, libellés via `find_local_label_by_name`/`get_local_label`, modèles via `list_templates` (comparaison par nom normalisé trim + minuscules).
  - `handle_export_rules(engine: &Engine) -> Result<Response, MiraError>`.

- [ ] **Step 1: Écrire `api_rules_export.rs`** (sur le modèle de `api_rules.rs` et de `support_rule_api`) :
  - `export_lists_rules_in_evaluation_order_with_enabled_state`
  - `export_writes_envelope_kind_rules_version_1`
  - `export_uses_portable_references` : le YAML contient l'adresse du compte, le nom du libellé, le nom du modèle ; aucun identifiant local.
  - `export_of_no_rule_is_valid_empty_list`
  - `export_emits_no_event`
  - `export_with_dangling_reference_fails_typed` : une règle dont le libellé a disparu fait échouer l'export avec une erreur typée, jamais un identifiant brut dans le fichier.

- [ ] **Step 2: Lancer pour voir l'échec** — `cargo test -p mira-core --test api_rules_export` ; attendu : erreur de compilation.

- [ ] **Step 3: Implémenter** types du rapport, variantes de contrat, `tx_refs.rs` et `export.rs`, ajouter `R::ExportRules` au `match` de `route` (et un bras provisoire explicite `R::ImportRules { .. }` rendant l'erreur `internal` « non implémenté » tant que la Task 3 n'est pas faite, pour que le `match` reste exhaustif).

- [ ] **Step 4: Instantanés de contrat** — exemples des deux requêtes et des deux réponses dans `rule_domain.rs` ; régénérer : `MIRA_UPDATE_SNAPSHOT=1 cargo test -p mira-core --test contract_snapshot_requests_and_responses --test contract_snapshot_events`, relire le diff de `tests/snapshots/contract_v1.json` (ajouts seulement). Étendre `contract_snapshot_rules.rs` à la forme des 2 requêtes et 2 réponses.

- [ ] **Step 5: Relancer** — `cargo test -p mira-core` (suites contrat et règles), `cargo test` dans `bridge/`, `cargo clippy --all-targets -- -D warnings`, `tools/check-norms`, `tools/check-architecture` ; attendu : vert.

- [ ] **Step 6: Commit**

```bash
git add engine docs
git commit -m "feat(règles): contrat d'export et d'import, résolution des références et export YAML"
```

---

### Task 3: Import, doublons, atomicité et idempotence

**Files:**
- Create: `mira-core/src/api/handlers/rule/{import,import_one}.rs`, `mira-core/src/storage/migrations/v014_rule_idempotency_result.sql`
- Modify: `mira-core/src/api/handlers/rule/{mod,idempotency}.rs`, `mira-core/src/storage/migrations/mod.rs`, `mira-core/src/storage/repositories/rules/idempotency.rs`, tests existants qui figent `schema_version()` à 13
- Test: `mira-core/tests/api_rules_import.rs`, `mira-core/tests/api_rules_import_idempotency.rs`, `mira-core/tests/storage_rule_idempotency.rs` (étendre)

**Interfaces:**
- Consumes: Tasks 1 et 2 (`envelope::unwrap`, `from_portable`, `impl RuleRefs for Tx`, `ImportReport`).
- Produces :
  - Migration `v014` : `ALTER TABLE rule_idempotency ADD COLUMN result TEXT` ; `RuleIdempotencyEntry` gagne `result: Option<String>` (JSON du rapport) ; mettre `None` aux appels existants.
  - `handle_import_rules(engine: &Engine, yaml: String, replace_existing: bool, key: IdempotencyKey) -> Result<Response, MiraError>`.
  - `import_one(tx, rule: Rule, replace_existing: bool) -> Result<ImportOutcome, MiraError>` dans `import_one.rs` : applique le doublon, `check_writable` (`rule/writing.rs`), `insert_rule` ou `update_rule`.

- [ ] **Step 1: Écrire `api_rules_import.rs`** :
  - `round_trip_on_blank_database_restores_identical_rules_and_order` (export d'une base A, import dans une base B qui a les mêmes comptes, libellés, modèles ; `list_rules` égal, ordre compris).
  - `existing_rule_is_reported_already_present_and_untouched` (défaut)
  - `replace_existing_overwrites_and_keeps_position`
  - `replace_with_unresolvable_reference_leaves_existing_rule_intact` (refus, pas de suppression préalable)
  - `each_refusal_reason_is_reported_with_position_and_name` : `unknown_account`, `unknown_mailbox`, `unknown_label`, `unknown_template`, `invalid_rule`.
  - `refused_rule_does_not_block_following_rules`
  - `duplicate_id_inside_file_refuses_second` (écrit une seule fois)
  - `new_rules_are_appended_after_existing_in_file_order`
  - `database_failure_mid_import_writes_nothing` (injecter l'échec comme dans `sync_rules_restart.rs`/tests d'atomicité existants)
  - `one_rule_changed_event_per_written_rule_and_none_otherwise` : import sans effet, rejeu, ou tout refusé → zéro événement.
  - `import_never_leaks_rule_content_in_errors_or_events` (`SECRET-MARK-42` dans un nom de règle et dans un motif).

- [ ] **Step 2: Écrire `api_rules_import_idempotency.rs`** :
  - `same_key_same_payload_replays_same_report_without_writing` (invariant 18)
  - `same_key_other_yaml_is_idempotency_conflict`
  - `same_key_other_replace_flag_is_idempotency_conflict` (Review Focus 5)

- [ ] **Step 3: Lancer pour voir l'échec** — `cargo test -p mira-core --test api_rules_import --test api_rules_import_idempotency` ; attendu : échec (import non implémenté).

- [ ] **Step 4: Implémenter** — la transaction (`engine.database.write`) enveloppe toute la boucle ; un refus est une valeur `ImportOutcome::Refused`, seule une erreur de base annule tout. Empreinte d'idempotence : `(replace_existing, SHA-256 du texte)` si une crate de hachage est déjà dépendance du workspace (`grep -n "sha2\|blake3" engine/Cargo.toml`), sinon le texte complet. Le rapport JSON est stocké dans `result` ; au rejeu on le relit et on le rend tel quel. Les événements `RuleChanged` sont émis après la transaction, un par règle `Imported` ou `Replaced`. Corriger les tests qui figent `schema_version()` à 13 (`grep -rn "schema_version" engine/crates`) : valeur attendue 14.

- [ ] **Step 5: Relancer** — `cargo test -p mira-core --test api_rules_import --test api_rules_import_idempotency --test storage_rule_idempotency` ; puis `cargo test -p mira-core` ; clippy, `tools/check-norms`, `tools/check-architecture` ; attendu : vert.

- [ ] **Step 6: Consigner les décisions** dans `docs/DECISIONS-AGENT.md` (contexte / choix / raison / retour arrière) : crate YAML, parcours par noms de champs, bornes, `id` conservés, migration v014.

- [ ] **Step 7: Commit**

```bash
git add engine docs
git commit -m "feat(règles): import YAML des règles (doublons, rapport, atomicité, idempotence)"
```

---

### Task 4: Interface Flutter

**Files:**
- Create: `app/lib/ui/rules_import_dialog.dart`
- Modify: `app/lib/engine/rule_admin.dart`, `app/lib/ui/rules_settings.dart`, `app/lib/i18n/` (textes, `engine_errors.dart`)
- Test: `app/test/rule_admin_export_import_test.dart`, `app/test/rules_import_dialog_test.dart`, `app/test/rules_settings_test.dart` (étendre), `app/test/events_contract_test.dart` (existant, doit rester vert)

**Interfaces:**
- Consumes: contrat JSON des Tasks 2 et 3 (domaine `rule`, ops `export_rules` et `import_rules`, `ImportOutcome` en snake_case).
- Produces :
  - `class ImportReport { List<ImportEntry> entries; int get imported; int get replaced; int get alreadyPresent; int get refused; fromJson }` ; `class ImportEntry { int position; String name; String outcome; String? reason }`.
  - `RuleAdmin.exportRules() -> Future<String>` ; `RuleAdmin.importRules(String yaml, {required bool replaceExisting}) -> Future<ImportReport>` (clé `ui-rules-import-…` non vide, différente à chaque appel).
  - `Future<bool?> showRulesImportDialog(BuildContext)` (renvoie l'état de la case « Remplacer les règles existantes », `null` si annulé) et `Widget`/fonction d'affichage du rapport.

- [ ] **Step 1: Écrire les tests** (faux `EngineClient` comme dans `rule_models_test.dart`) :
  - `rule_admin_export_import_test.dart` : `exportRules` envoie `domain: rule`, `op: export_rules` et rend le texte ; `importRules` envoie `yaml`, `replace_existing` et une clé non vide différente à chaque appel ; décodage du rapport (compteurs, raison d'un refus).
  - `rules_import_dialog_test.dart` : case décochée par défaut ; la valeur cochée est renvoyée ; Annuler renvoie `null` ; le rapport affiche « N importées, N déjà présentes, N refusée(s) » et la raison traduite de chaque refus avec position et nom.
  - `rules_settings_test.dart` : les boutons Exporter et Importer sont présents ; export appelle `saveFile` avec un nom `mira-regles-AAAA-MM-JJ.yaml` (injecter la date et le sélecteur de fichier comme dans les tests de `store.dart`) ; import lit le fichier, ouvre la boîte, appelle le moteur puis affiche le rapport ; une erreur de fichier entier (`invalid_input.yaml_syntax`) s'affiche traduite avec ligne et colonne.

- [ ] **Step 2: Lancer pour voir l'échec** — `cd app && flutter test test/rule_admin_export_import_test.dart` ; attendu : méthodes absentes.

- [ ] **Step 3: Implémenter** — `rule_admin.dart` (`_send` existant), boîte et rapport dans `rules_import_dialog.dart` (fichier séparé, `rules_settings.dart` ne fait que le branchement), sélecteurs `FilePicker.saveFile` et `FilePicker.pickFiles` (extensions `yaml`, `yml`, lecture des octets pour Android), textes dans `i18n`, codes d'erreur dans `i18n/engine_errors.dart` : `invalid_input.yaml_syntax`, `export_version_unsupported`, `export_kind_mismatch`, `export_too_large`, et les cinq raisons de refus. `i18n_coverage_test` doit rester vert.

- [ ] **Step 4: Relancer** — `flutter test test/rule_admin_export_import_test.dart test/rules_import_dialog_test.dart test/rules_settings_test.dart test/events_contract_test.dart test/i18n_coverage_test.dart`, puis la suite complète `flutter test` ; `flutter analyze` ; attendu : vert.

- [ ] **Step 5: Commit**

```bash
git add app
git commit -m "feat(app): export et import YAML des règles (boutons, boîte d'import, rapport)"
```

---

## Fin de parcours

Après la tâche 4 : demander à l'utilisateur l'accord pour lancer `./ci.sh`, puis un relecteur indépendant sur la branche entière avant fusion dans `dev`. Les libellés locaux, puis les modèles, auront chacun leur spec et leur plan, en réutilisant `portability::envelope` (`kind: local_labels`, `kind: message_templates`).

## Auto-relecture

- **Couverture du spec :** format et enveloppe (T1), références portables et refus par règle (T1, T3), doublons et option de remplacement (T3), contrat, rapport, erreurs de fichier entier (T1, T2), bornes (T1), atomicité et idempotence (T3), événements (T3), confidentialité (T1, T3), interface (T4). Le tri « position » des règles ajoutées est dans T3.
- **Écarts au spec, assumés :** (1) l'événement est `RuleChanged` par règle écrite, non un `RulesChanged` inexistant ; (2) le rejeu d'une clé rend le rapport d'origine, ce qui exige la colonne `result` (migration v014) que le spec ne mentionnait pas ; (3) un fichier sans `schema_version` est refusé comme règle invalide, car le contrôle des champs inconnus compare l'entrée à sa resérialisation.
- **Cohérence des types :** `RuleRefs`, `PortableError`, `ImportReport`/`ImportEntry`/`ImportOutcome`/`ImportRefusal` sont définis en T1/T2 et réutilisés tels quels en T3/T4 ; côté Dart `export_rules`, `import_rules`, `replace_existing`, `entries`.
