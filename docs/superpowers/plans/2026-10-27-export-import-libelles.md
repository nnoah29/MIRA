# Export et import des libellés locaux en YAML — plan d'implémentation

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Exporter les libellés locaux (nom et couleur) dans un fichier YAML et les réimporter, avec rapport par libellé, doublons ignorés ou remplacés, sans toucher à leur pose sur les conversations.

**Architecture:** Même mécanisme que pour les règles (AG-81) : le moteur convertit du texte YAML derrière `portability::envelope` (`kind: local_labels`), l'app lit et écrit le fichier. Deux opérations s'ajoutent au domaine `local_label` (`ExportLocalLabelsFile`, `ImportLocalLabelsFile`) ; le rapport rejoué par clé passe par `LabelCommand` (aucune migration). La boîte d'options et le rapport Flutter deviennent génériques et sont partagés avec les règles.

**Tech Stack:** Rust 2024 (workspace `engine/`, toolchain 1.99.0), `yaml_serde` déjà présent ; Flutter/Dart (`app/`), `cargo test` / `flutter test`.

**Spec:** `docs/superpowers/specs/2026-10-27-export-import-libelles-design.md`

## Global Constraints

- Branche `feat/yaml-import-export` ; commits à l'identité de l'utilisateur, **sans trailer d'attribution**. Ne pas ajouter les icônes Android ni `packaging/VERSION` aux commits (`git add` explicite, jamais `git add app` ni `git add .`).
- Pas de `./ci.sh` sans l'accord de l'utilisateur : vérifier avec `cargo test -p <crate> --test <fichier>`, `cargo clippy`, `tools/check-norms`, `flutter test <fichier>`, puis demander.
- Normes (`tools/check-norms`, tests compris) : fonction ≤ 30 lignes (**après `cargo fmt`**), fichier ≤ 6 fonctions avec corps, aucun commentaire dans un corps de fonction, une ligne `///` en français avant chaque `fn`, `//!` de 1 à 3 lignes en tête de fichier, identifiants en anglais. Pas de `unsafe`, pas de `unwrap`/`expect`/`panic!` hors tests. Une impl de trait compte ses méthodes dans le plafond de 6.
- Enveloppe : `portability::envelope::{wrap, unwrap}` avec `kind = "local_labels"`, `items_key = "labels"` ; mêmes bornes (2 Mo, 500 éléments), ancres refusées, mêmes erreurs de fichier entier. Ne pas modifier l'enveloppe.
- Format : `labels: [{ name: <texte>, color: "#AARRGGBB" }]` ; à l'import `#RRGGBB` (opaque, alpha `FF`) ou `#AARRGGBB`, hexadécimal insensible à la casse ; `color` absent → `default_color(existants + déjà ajoutés)` ; tout autre champ → `InvalidLabel`.
- Nom : `clean_name` (non vide, ≤ 64 octets après trim) ; unicité par clé de nom (trim + minuscules). Plafond `MAX_LABELS = 200` libellés au total.
- Doublon (nom déjà pris) : « déjà présent » par défaut ; avec `replace_existing`, le libellé existant prend le nom (casse comprise) et la couleur du fichier ; **un remplacement qui ne change ni le nom ni la couleur est « déjà présent »**, sans écriture. Doublon interne au fichier : le second est refusé `DuplicateInFile`.
- Un libellé refusé n'empêche pas les suivants ; une seule transaction ; rien d'écrit si la base échoue.
- Nom du rapport tronqué à 64 caractères (au plus) ; aucun nom de libellé dans les journaux, erreurs ni événements.
- Événement `EventKind::LocalLabelsChanged` : une seule fois après un import qui a écrit quelque chose ; jamais en lecture, rejeu ou import sans effet.
- Empreinte d'idempotence : `(replace_existing, SHA-256 hexadécimal du texte)`.
- L'opération existante `ImportLocalLabels` (reprise des libellés de l'ancienne app) et le type `ImportReport` existant ne changent pas.

## Review Focus

1. `compta` et `Compta` dans le même fichier : second refusé, un seul libellé écrit. (Task 3)
2. 190 libellés existants, fichier de 20 nouveaux : 10 importés, 10 refusés `TooManyLabels`. (Task 3)
3. Couleur `#FFF`, `rouge`, `#GGGGGG`, entier non texte : libellé refusé `InvalidColor`, les autres importés. (Tasks 1, 3)
4. Nom de 1,9 Mo dans une entrée refusée : rapport tronqué, rien d'énorme mémorisé. (Task 3)
5. Réimporter sa propre sauvegarde avec « remplacer » : tout « déjà présent », zéro événement. (Task 3)
6. Fichier mêlant libellés valides et un champ inconnu (`couleur:` pour `color:`) : seul ce libellé est refusé `InvalidLabel`. (Tasks 1, 3)

---

## File Structure

| Fichier | Responsabilité |
|---|---|
| `mira-core/src/portability/labels/{mod,color,to_portable,from_portable}.rs` (créer) | forme portable pure d'un libellé : couleur `#AARRGGBB`, lecture stricte |
| `mira-vocabulary/src/domain/label_import_report.rs` (créer) | `LabelImportReport`, `LabelImportEntry`, `LabelImportOutcome`, `LabelImportRefusal` |
| `mira-core/src/api/handlers/local_label/{export_file,import_file,import_file_one}.rs` (créer) | un handler par opération |
| `mira-core/src/api/contract/{local_label_requests,local_label_responses}.rs` (modifier) | deux requêtes, deux réponses |
| `app/lib/ui/import_dialog.dart`, `app/lib/ui/import_report.dart` (renommés depuis `rules_import_*`) | boîte d'options et rapport génériques |
| `app/lib/engine/local_label_admin.dart`, `app/lib/ui/labels_settings.dart` (modifier) | côté Flutter des libellés |

---

### Task 1: Forme portable d'un libellé (fonctions pures)

**Files:**
- Create: `mira-core/src/portability/labels/{mod,color,to_portable,from_portable}.rs`
- Modify: `mira-core/src/portability/mod.rs`
- Test: `mira-core/tests/portability_labels_color.rs`, `mira-core/tests/portability_labels_format.rs`

**Interfaces:**
- Produces :
  - `portability::labels::format_color(color: u32) -> String` : `"#AARRGGBB"` en minuscules, 8 chiffres.
  - `portability::labels::parse_color(text: &str) -> Option<u32>` : `#RRGGBB` (alpha `FF`) ou `#AARRGGBB`, insensible à la casse ; toute autre forme → `None`.
  - `portability::labels::to_portable(label: &LocalLabel) -> serde_json::Value` : `{ "name": …, "color": "#aarrggbb" }`.
  - `struct PortableLabel { name: String, color: Option<u32> }` (`Debug, Clone, PartialEq, Eq`) et `enum PortableLabelError { InvalidColor, Invalid }` (`Debug, Clone, Copy, PartialEq, Eq`).
  - `portability::labels::from_portable(value: serde_json::Value) -> Result<PortableLabel, PortableLabelError>` : objet avec `name` (texte) obligatoire et `color` (texte) facultatif ; une `color` mal formée ou non textuelle → `InvalidColor` ; un autre champ, un objet sans `name` ou un `name` non textuel → `Invalid`. Le nom n'est pas contrôlé ici (Task 3).

- [ ] **Step 1: Écrire `portability_labels_color.rs`** (appels directs) :
  - `format_is_lowercase_eight_digits` : `format_color(0xFF1E90FF) == "#ff1e90ff"`, `format_color(0) == "#00000000"`.
  - `parse_accepts_rrggbb_as_opaque` : `parse_color("#1e90ff") == Some(0xFF1E90FF)`.
  - `parse_accepts_aarrggbb_any_case` : `parse_color("#80FFAA00") == Some(0x80FFAA00)` et en minuscules.
  - `parse_round_trips_every_format` : pour `[0, 1, 0x80FFAA00, 0xFFFFFFFF, 0xFFE8542B]`, `parse_color(&format_color(c)) == Some(c)`.
  - `parse_refuses_other_forms` (Review Focus 3) : `"#FFF"`, `"FFF000"`, `"rouge"`, `"#GGGGGG"`, `"#1234567"`, `"#123456789"`, `""`, `"#"` → `None`.

- [ ] **Step 2: Lancer pour voir l'échec** — `cd engine && cargo test -p mira-core --test portability_labels_color` ; attendu : erreur de compilation (module `labels` absent).

- [ ] **Step 3: Implémenter `color.rs`** (`format_color`, `parse_color` ; `u32::from_str_radix` sur 6 ou 8 chiffres hexadécimaux ASCII, sans `+`), `mod.rs` et la déclaration `pub mod labels;`.

- [ ] **Step 4: Relancer** — 5 tests PASS.

- [ ] **Step 5: Écrire `portability_labels_format.rs`** :
  - `round_trip_keeps_name_and_color` : `from_portable(to_portable(&label))` rend `PortableLabel { name, color: Some(label.color) }`.
  - `color_is_optional` : `{"name": "Voyages"}` → `color: None`.
  - `invalid_color_is_reported_separately` (Review Focus 3) : `{"name":"A","color":"rouge"}` et `{"name":"A","color":12}` → `InvalidColor`.
  - `unknown_field_is_invalid` (Review Focus 6) : `{"name":"A","couleur":"#ff0000"}` → `Invalid`.
  - `missing_or_non_text_name_is_invalid` : `{}`, `{"name": 3}`, `"texte"`, `[1]` → `Invalid`.

- [ ] **Step 6: Implémenter `to_portable.rs` et `from_portable.rs`** ; la lecture compte les clés de l'objet pour refuser tout champ hors `name` / `color`.

- [ ] **Step 7: Relancer** — `cargo test -p mira-core --test portability_labels_color --test portability_labels_format` ; `cargo fmt` ; `cargo clippy -p mira-core --all-targets -- -D warnings` ; `tools/check-norms engine/crates` ; attendu : vert.

- [ ] **Step 8: Commit**

```bash
git add engine/crates
git commit -m "feat(libellés): forme portable d'un libellé et couleur #AARRGGBB"
```

---

### Task 2: Contrat, vocabulaire et export

**Files:**
- Create: `mira-vocabulary/src/domain/label_import_report.rs`, `mira-core/src/api/handlers/local_label/export_file.rs`
- Modify: `mira-vocabulary/src/{domain/mod.rs,lib.rs}`, `mira-core/src/api/contract/{local_label_requests,local_label_responses,mod}.rs`, `mira-core/src/api/mod.rs`, `mira-core/src/api/handlers/local_label/mod.rs`
- Test: `mira-core/tests/api_local_labels_export.rs`, `mira-core/tests/contract_examples/local_label_domain.rs` (+ `snapshot.rs`/`mod.rs` si nécessaire), `mira-core/tests/contract_snapshot_local_labels.rs`

**Interfaces:**
- Consumes: Task 1 (`to_portable`), `portability::envelope::wrap`.
- Produces :
  - Vocabulaire (tous `Debug, Clone, PartialEq, Eq, Serialize, Deserialize`, `snake_case`, énumérations étiquetées `type`) : `LabelImportRefusal` ∈ `InvalidName`, `InvalidColor`, `InvalidLabel`, `DuplicateInFile`, `TooManyLabels` ; `LabelImportOutcome` ∈ `Imported`, `Replaced`, `AlreadyPresent`, `Refused { reason: LabelImportRefusal }` ; `LabelImportEntry { position: u32, name: String, outcome: LabelImportOutcome }` ; `LabelImportReport { entries: Vec<LabelImportEntry> }`. Ré-exportés par `mira_vocabulary`.
  - Contrat : `LocalLabelRequest::ExportLocalLabelsFile`, `LocalLabelRequest::ImportLocalLabelsFile { yaml: String, replace_existing: bool, idempotency_key: IdempotencyKey }` ; `LocalLabelResponse::LabelsFileExport { yaml: String }`, `LocalLabelResponse::LabelsFileImport(LabelImportReport)`.
  - `handle_export_labels_file(engine: &Engine) -> Result<Response, MiraError>` : tous les libellés de `list_local_labels`, enveloppés `wrap("local_labels", "labels", …)`.

- [ ] **Step 1: Écrire `api_local_labels_export.rs`** (modèle : `api_local_labels.rs` et `support_label_api`) ; ajouter à `support_label_api/calls.rs` une fonction `export_yaml(engine) -> Result<String, String>` (file séparé `support_label_api/files.rs` si le plafond de 6 fonctions l'exige) :
  - `export_lists_labels_with_name_and_hex_color` : trois libellés créés avec couleurs connues ; `unwrap(&yaml, "local_labels", "labels")` rend trois éléments, nom et `#aarrggbb` exacts, dans l'ordre de `ListLocalLabels`.
  - `export_writes_envelope_kind_local_labels_version_1`
  - `export_of_no_label_is_valid_empty_list`
  - `export_emits_no_event`
  - `export_contains_no_conversation_or_id` : le YAML ne contient ni l'identifiant d'un libellé ni `thread`.

- [ ] **Step 2: Lancer pour voir l'échec** — `cargo test -p mira-core --test api_local_labels_export` ; attendu : erreur de compilation.

- [ ] **Step 3: Implémenter** types du rapport, variantes de contrat, `export_file.rs`, bras `R::ExportLocalLabelsFile` du `match` de `route` et un bras provisoire explicite `R::ImportLocalLabelsFile { .. }` rendant l'erreur `internal.bug` « import des libellés non implémenté » (le `match` reste exhaustif jusqu'à la Task 3).

- [ ] **Step 4: Instantanés de contrat** — exemples des deux requêtes et des deux réponses (un rapport avec un sort de chaque sorte, dont deux refus) dans `local_label_domain.rs` ; adapter le décompte d'opérations de `contract_snapshot_local_labels.rs` ; régénérer : `MIRA_UPDATE_SNAPSHOT=1 cargo test -p mira-core --test contract_snapshot_requests_and_responses`, puis **relire le diff** de `tests/snapshots/contract_v1.json` (ajouts seulement).

- [ ] **Step 5: Relancer** — `cargo test -p mira-core` (toutes les suites), `cd bridge && cargo test`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt`, `tools/check-norms engine/crates`, `tools/check-architecture engine` ; attendu : vert. Exécuter le test complet en arrière-plan, sa durée dépasse deux minutes.

- [ ] **Step 6: Commit**

```bash
git add engine/crates bridge/Cargo.lock
git commit -m "feat(libellés): contrat d'export et d'import des libellés et export YAML"
```

---

### Task 3: Import, doublons, plafond, atomicité et idempotence

**Files:**
- Create: `mira-core/src/api/handlers/local_label/{import_file,import_file_one}.rs`
- Modify: `mira-core/src/api/handlers/local_label/mod.rs`
- Test: `mira-core/tests/api_local_labels_import_file.rs`, `mira-core/tests/api_local_labels_import_file_rules.rs`, `mira-core/tests/api_local_labels_import_file_effects.rs`, `mira-core/tests/api_local_labels_import_file_idempotency.rs`

**Interfaces:**
- Consumes: Tasks 1 et 2 (`from_portable`, `parse_color`, `envelope::unwrap`, types du rapport) ; existant : `clean_name`, `ensure_room`-style plafond (`MAX_LABELS`), `default_color`, `announce`, `LabelCommand::{new, replay::<T>, record}`, `Tx::{insert_local_label, rename_local_label, recolor_local_label, find_local_label_by_name, list_local_labels}`.
- Produces :
  - `handle_import_labels_file(engine: &Engine, yaml: String, replace_existing: bool, key: IdempotencyKey) -> Result<Response, MiraError>` : `unwrap` d'abord (erreurs de fichier entier avant toute clé), puis `LabelCommand::new(key, "import_labels_file", &(replace_existing, digest))`, une transaction, `replay::<LabelImportReport>` en tête, `record` du rapport en fin ; `announce` une fois si au moins un libellé a été écrit.
  - `import_label(tx, value: Value, replace: bool, seen: &mut HashSet<String>, count: &mut usize) -> Result<LabelImportOutcome, MiraError>` dans `import_file_one.rs` : lecture (`from_portable`), nom (`clean_name`), doublon interne (clé de nom dans `seen`), existant par `find_local_label_by_name`, plafond (`count` = libellés existants + ajoutés, refus `TooManyLabels` quand il atteint `MAX_LABELS`), puis insertion ou remplacement ; `rename_local_label` + `recolor_local_label` uniquement si le nom (casse comprise) ou la couleur diffère.
  - Le nom du rapport est tronqué à 64 caractères au plus (par caractères entiers).

- [ ] **Step 1: Écrire `api_local_labels_import_file.rs`** (aller-retour et doublons) :
  - `round_trip_restores_names_and_colors` : trois libellés, export, suppression des trois, import ; `labels()` rend les mêmes (nom, couleur) (les identifiants sont neufs) ; sorts `Imported` ×3.
  - `existing_label_is_reported_already_present_and_untouched` : libellé recoloré après l'export, import par défaut → `AlreadyPresent`, couleur inchangée.
  - `replace_existing_updates_name_case_and_color` : `compta` existant, fichier `Compta` + autre couleur, `replace_existing = true` → `Replaced`, nom `Compta`, couleur du fichier, même identifiant.
  - `replace_with_identical_label_is_already_present_without_event` (Review Focus 5) : réimport de sa propre sauvegarde avec `replace_existing = true` → tous `AlreadyPresent`, `changed_count` inchangé.
  - `labels_are_matched_ignoring_case_and_edge_spaces` : fichier `"  COMPTA "` vs libellé `Compta` → `AlreadyPresent`.

- [ ] **Step 2: Écrire `api_local_labels_import_file_rules.rs`** (refus, plafond) :
  - `duplicate_name_inside_file_refuses_second` (Review Focus 1) : `compta` puis `Compta` → `[Imported, Refused(DuplicateInFile)]`, un seul libellé écrit.
  - `invalid_name_is_refused_and_others_imported` : nom vide, nom de 65 caractères → `InvalidName`, libellé suivant importé.
  - `invalid_color_and_unknown_field_are_refused` (Review Focus 3 et 6) : `rouge`, `#FFF`, `couleur:` → `InvalidColor` / `InvalidColor` / `InvalidLabel`, les autres importés.
  - `color_defaults_when_missing` : `{name: Voyages}` importé avec la couleur par défaut de la palette, pas `0`.
  - `cap_of_200_refuses_the_overflow` (Review Focus 2) : 190 libellés créés, fichier de 20 nouveaux → 10 `Imported`, 10 `Refused(TooManyLabels)`, exactement 200 libellés ensuite.
  - `oversized_name_in_report_is_truncated` (Review Focus 4) : nom refusé de 100 000 caractères → le nom du rapport fait au plus 64 caractères.

- [ ] **Step 3: Écrire `api_local_labels_import_file_effects.rs`** :
  - `one_event_after_a_write_and_none_otherwise` : un import qui écrit → +1 `LocalLabelsChanged` ; rejeu de la même clé, import sans effet → +0.
  - `invalid_file_writes_nothing_and_emits_nothing` : syntaxe invalide → `invalid_input.yaml_syntax`, aucun libellé, aucun événement.
  - `database_failure_mid_import_writes_nothing` : déclencheur SQL `BEFORE INSERT ON local_labels WHEN NEW.name = 'Boom'` posé par une connexion `rusqlite` sur `ProfilePaths::resolve(...).database` ; fichier `[Un, Boom]` → erreur, zéro libellé ajouté.
  - `import_never_leaks_label_names_in_errors_or_events` : marqueur `SECRET-MARK-42` dans un nom accepté et dans un fichier de syntaxe invalide ; absent des erreurs et des événements.

- [ ] **Step 4: Écrire `api_local_labels_import_file_idempotency.rs`** :
  - `same_key_same_payload_replays_same_report_without_writing` (invariant 18)
  - `same_key_other_yaml_is_idempotency_conflict`
  - `same_key_other_replace_flag_is_idempotency_conflict`

- [ ] **Step 5: Lancer pour voir l'échec** — `cargo test -p mira-core --test api_local_labels_import_file --test api_local_labels_import_file_rules --test api_local_labels_import_file_effects --test api_local_labels_import_file_idempotency` ; attendu : échecs `internal.bug` (import non implémenté).

- [ ] **Step 6: Implémenter** `import_file.rs` et `import_file_one.rs`, brancher `R::ImportLocalLabelsFile { yaml, replace_existing, idempotency_key: key }` dans `route`. Le plafond compte les libellés existants (`list_local_labels().len()`) puis chaque libellé ajouté ; un remplacement ne consomme pas de place.

- [ ] **Step 7: Relancer** — les quatre suites PASS ; puis `cargo test -p mira-core -p mira-vocabulary` en arrière-plan, `cargo clippy --all-targets -- -D warnings`, `cargo fmt`, `tools/check-norms engine/crates`, `tools/check-architecture engine`, `tools/check-forbidden-patterns` ; attendu : vert.

- [ ] **Step 8: Consigner la décision** dans `docs/DECISIONS-AGENT.md` (AG-82 : export et import des libellés ; format `#AARRGGBB`, remplacement identique = déjà présent, plafond, pas de pose, aucune migration ; contexte / choix / limites / retour arrière).

- [ ] **Step 9: Commit**

```bash
git add engine/crates docs/DECISIONS-AGENT.md
git commit -m "feat(libellés): import YAML des libellés (doublons, plafond, rapport, atomicité, idempotence)"
```

---

### Task 4: Interface Flutter

**Files:**
- Rename: `app/lib/ui/rules_import_dialog.dart` → `app/lib/ui/import_dialog.dart`, `app/lib/ui/rules_import_report.dart` → `app/lib/ui/import_report.dart`, `app/lib/data/rule_import.dart` → `app/lib/data/file_import.dart` (types `RuleImportEntry` / `RuleImportReport` → `FileImportEntry` / `FileImportReport`, `RulesExport` inchangé), et `app/test/rules_import_dialog_test.dart` → `app/test/import_dialog_test.dart` (changements d'identifiants et de chemins d'import seulement)
- Modify: `app/lib/engine/{rule_admin,local_label_admin}.dart`, `app/lib/ui/{rules_settings,labels_settings,rules_file_flow}.dart`, `app/lib/i18n/lang/en.dart`
- Test: `app/test/local_label_export_import_test.dart`, `app/test/labels_settings_file_test.dart`, `app/test/import_dialog_test.dart`, `app/test/rules_settings_file_test.dart` (existant, doit rester vert)

**Interfaces:**
- Consumes: contrat JSON des Tasks 2 et 3 (domaine `local_label`, ops `export_local_labels_file` et `import_local_labels_file`, `LabelsFileExport` → `{yaml}`, `LabelsFileImport` → `{entries: [{position, name, outcome: {type, reason?: {type}}}]}`).
- Produces :
  - `showImportOptionsDialog(BuildContext, {required String title, required String note, required String replaceLabel}) -> Future<bool?>` ; `showRulesImportDialog(context)` reste un enrobage fixe (textes des règles, inchangés).
  - `FileImportReportView({required FileImportReport report, required String Function(FileImportEntry) refusal})` et `showFileImportReport(context, report, refusal)` ; `rulesRefusalText(entry)` (l'ancien `refusalText`) et `labelsRefusalText(entry)` : `invalid_name` → « Nom invalide », `invalid_color` → « Couleur invalide », `invalid_label` → « Libellé illisible », `duplicate_in_file` → « Nom déjà présent plus haut dans le fichier », `too_many_labels` → « Trop de libellés ».
  - `LocalLabelAdmin.exportLabelsFile() -> Future<String>` ; `LocalLabelAdmin.importLabelsFile(String yaml, {bool replaceExisting = false}) -> Future<FileImportReport>` (clé `ui-label-import-…` neuve à chaque appel).
  - `LabelsSettings({super.key, this.saveFile = saveWithPicker, this.pickFile = pickYamlWithPicker, this.clock = DateTime.now})`, boutons `ValueKey('export-labels')` et `ValueKey('import-labels')`, désactivés (`onTap: null`) pendant une opération.
  - `labelsExportName(DateTime) -> String` (`mira-libelles-AAAA-MM-JJ.yaml`) dans `rules_file_flow.dart`, à côté de `rulesExportName`.

- [ ] **Step 1: Écrire les tests** (modèle : `rule_admin_export_import_test.dart`, `rules_settings_file_test.dart`) :
  - `local_label_export_import_test.dart` : `exportLabelsFile` envoie `domain: local_label`, `op: export_local_labels_file` et rend le texte ; `importLabelsFile` envoie `yaml`, `replace_existing` et une clé non vide, différente à chaque appel ; décodage des compteurs et d'un refus `too_many_labels` (position, nom).
  - `import_dialog_test.dart` : mêmes cas qu'avant (case décochée par défaut, cochée transmise, Annuler renvoie `null`, rapport) avec le titre et la case fournis par l'appelant ; un second jeu de textes (libellés) s'affiche sans mention des règles.
  - `labels_settings_file_test.dart` : Exporter enregistre le texte sous `mira-libelles-2026-10-27.yaml` ; Importer lit, ouvre la boîte (« Remplacer les libellés existants »), appelle le moteur, montre le rapport ; aucun fichier choisi → aucun appel ; erreur `invalid_input.yaml_syntax` affichée avec « ligne 3, colonne 5 » ; **les deux boutons sont désactivés pendant l'import** (le test garde l'import en attente avec un `Completer`).
  - `rules_settings_file_test.dart` et `rule_admin_export_import_test.dart` : adaptés aux noms `FileImport*` seulement.
  - `rules_file_flow` : `pickYamlWithPicker` borne la taille **après** lecture quand `length()` est inconnu (fonction découplée testable : `Future<String?> readYamlFile(PlatformFile-like)` ou test sur une fonction pure `ensureRulesFileSize(int bytes)` qui lève `invalid_input.export_too_large` au-delà de 2 Mo).

- [ ] **Step 2: Lancer pour voir l'échec** — `cd app && flutter test test/local_label_export_import_test.dart` ; attendu : méthodes absentes.

- [ ] **Step 3: Renommer et généraliser** (`git mv`, puis adapter les imports) ; rendre la boîte d'options et le rapport paramétrables ; garder les clés `import-replace`, `import-confirm`, `import-cancel`.

- [ ] **Step 4: Implémenter** `LocalLabelAdmin.exportLabelsFile/importLabelsFile`, le garde « occupé » et les deux boutons dans `labels_settings.dart` (la logique de fichier est dans `rules_file_flow.dart` : `exportNamedFile`/`importYamlText` partagés avec les règles, `labels_settings.dart` ne fait que le branchement), `ensureRulesFileSize`, textes `i18n` (français + anglais) et les raisons de refus des libellés. Vérifier qu'aucune clé de traduction n'est dupliquée dans `en.dart`.

- [ ] **Step 5: Relancer** — `flutter test test/local_label_export_import_test.dart test/import_dialog_test.dart test/labels_settings_file_test.dart test/rules_settings_file_test.dart test/rule_admin_export_import_test.dart test/i18n_coverage_test.dart test/local_label_admin_test.dart`, `dart format lib test`, `flutter analyze`, puis la suite complète `flutter test` en arrière-plan ; attendu : vert, hors l'échec préexistant `about_settings_test` (`appVersion` 1.1.6 contre `pubspec` 1.2.0, à signaler et non à corriger ici).

- [ ] **Step 6: Commit**

```bash
git add app/lib app/test
git commit -m "feat(app): export et import YAML des libellés locaux, boîte d'import générique"
```

---

## Fin de parcours

Après la tâche 4 : demander à l'utilisateur l'accord pour lancer `./ci.sh`, puis un relecteur indépendant (agent `relecteur`, modèle le plus capable) sur la branche entière, en lui donnant le Review Focus et les rulings. Les modèles de messages suivront avec leur propre spec et leur propre plan (`kind: message_templates`).

## Auto-relecture

- **Couverture du spec :** format et couleurs (T1), contrat, rapport et export (T2), import, doublons, doublon interne, plafond, atomicité, idempotence, événements, confidentialité, troncature du nom (T3), interface générique, gardes de l'app (T4). « Remplacement identique = déjà présent » (T3). Aucune migration (T3 : `LabelCommand`).
- **Écarts au spec, assumés :** aucun nouvel écart. La suppression des mineurs M8 (taille inconnue) et M9 (garde « occupé ») ne s'applique qu'aux libellés et à la fonction de lecture partagée ; l'écran des règles garde son comportement actuel pour M9.
- **Cohérence des types :** `PortableLabel`, `PortableLabelError`, `LabelImportReport/Entry/Outcome/Refusal` définis en T1/T2 et réutilisés tels quels en T3 ; côté Dart `FileImportReport/Entry`, `export_local_labels_file`, `import_local_labels_file`, `replace_existing`, `entries`.
- **Proportion :** quatre tâches, comme pour les règles ; les corps de fonction sont laissés à l'implémenteur, le plan ne fixe que signatures, valeurs de la spec et noms de tests.
