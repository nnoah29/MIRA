# Modèles de messages — plan d'implémentation

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ajouter des modèles de messages locaux (nom, objet, corps) avec variables `{{…}}` résolues par le moteur, gérés dans un écran de réglages et insérés depuis l'éditeur (bouton + raccourci `/`).

**Architecture:** Calqué sur les libellés locaux (AG-73 à AG-75) : entité dans `mira-vocabulary`, migration SQLite + dépôt + module de rendu pur dans `mira-core`, domaine d'API `Template` (un handler par opération, idempotence sur la création, événement `TemplatesChanged`), puis couche Dart `template_admin` + état + écran + intégration au composeur. Le pont FFI transporte l'enveloppe existante : aucune fonction nouvelle.

**Tech Stack:** Rust 2024 (workspace `engine/`, toolchain 1.99.0, rusqlite, serde), Flutter/Dart (`app/`), tests `cargo test` / `flutter test`.

**Spec:** `docs/superpowers/specs/2026-10-25-modeles-de-messages-design.md`

## Global Constraints

- Variables, syntaxe `{{nom}}`, espaces tolérés : `contact.nom`, `contact.prénom`, `contact.email`, `date`, `heure`, `moi.nom`, `moi.email`, `objet`.
- Variable inconnue, ou sans valeur (absente ou chaîne vide) : **laissée telle quelle** et listée dans `unresolved`.
- Modèles locaux et globaux : aucune synchro, aucun rattachement à un compte.
- Toute la logique dans le moteur Rust ; Flutter n'affiche et n'appelle que.
- Le cœur ne nomme aucun provider ; pas de `unsafe`, pas de `unwrap`/`expect`/`panic!` hors tests.
- Normes (`tools/check-norms`) : fonction ≤ 30 lignes, fichier ≤ 6 fonctions avec corps (tests compris, tests en fichiers séparés), aucun commentaire dans un corps de fonction, une ligne `///` en français avant chaque `fn`, `//!` de 1 à 3 lignes en tête de fichier. Identifiants en anglais.
- Pas de `./ci.sh` sans l'accord de l'utilisateur : vérifier avec `cargo test -p <crate> --test <fichier>`, `cargo clippy`, `flutter test <fichier>`, puis demander.
- Commits à l'identité de l'utilisateur, **sans trailer d'attribution**. Branche : `feat/message-templates`.
- Décisions prises par ce plan (à consigner dans `docs/DECISIONS-AGENT.md`, numéro suivant la séquence) : (a) `RenderContext` porte `date` et `time` **déjà formatées** par l'app (pas de fuseau dans le moteur) ; (b) bornes : `name` ≤ 100 caractères, `subject` ≤ 300, `body` ≤ 100 000 ; (c) une seule passe de remplacement.

## Review Focus

1. Nom « Merci » puis « merci » (ou avec espaces de bord) : refusé comme doublon, comme pour les libellés (`name_key` = minuscules + trim).
2. Corps au-delà de 100 000 caractères, nom > 100, objet > 300 : refusés avec une erreur typée `invalid_input.template_too_long`, rien d'écrit.
3. Valeur de variable contenant elle-même `{{date}}` (ex. nom de contact `{{date}}`) : insérée telle quelle, jamais re-substituée (une seule passe).
4. Même variable répétée, ou même inconnue répétée : `unresolved` la liste une fois, dans l'ordre de première apparition.
5. Valeur présente mais vide (`contact.nom = ""`) : traitée comme manquante, variable laissée visible.
6. Modèle supprimé pendant que le sélecteur est ouvert : l'insertion montre l'erreur `not_found.template` traduite et la liste se rafraîchit.

---

## File Structure

| Fichier | Responsabilité |
|---|---|
| `engine/crates/mira-vocabulary/src/domain/message_template.rs` (créer) | `MessageTemplate`, `RenderContext`, `ContactInfo`, `RenderedTemplate` |
| `engine/crates/mira-vocabulary/src/ids/kinds.rs` (modifier) | `TemplateId` |
| `engine/crates/mira-vocabulary/src/error/codes/{invalid_input,not_found}.rs` (modifier) | `TEMPLATE_NAME_EMPTY`, `TEMPLATE_NAME_TAKEN`, `TEMPLATE_TOO_LONG`, `TEMPLATE` |
| `engine/crates/mira-core/src/storage/migrations/v012_message_templates.sql` (créer) | tables `message_templates`, `template_idempotency` |
| `engine/crates/mira-core/src/storage/repositories/message_templates/` (créer) | `mod`, `rows`, `read`, `write`, `idempotency` |
| `engine/crates/mira-core/src/templates/` (créer) | `mod`, `variables` (enum + parse), `render` (fonction pure) |
| `engine/crates/mira-core/src/api/contract/template_{requests,responses}.rs` (créer) | contrat |
| `engine/crates/mira-core/src/api/handlers/template/` (créer) | un fichier par opération |
| `app/lib/engine/template_admin.dart`, `app/lib/state/templates.dart`, `app/lib/ui/templates_settings.dart`, `app/lib/ui/template_picker.dart` (créer) | côté Flutter |

---

### Task 1: Vocabulaire, stockage et rendu des variables

**Files:**
- Create: `mira-vocabulary/src/domain/message_template.rs`, `mira-core/src/storage/migrations/v012_message_templates.sql`, `mira-core/src/storage/repositories/message_templates/{mod,rows,read,write,idempotency}.rs`, `mira-core/src/templates/{mod,variables,render}.rs`
- Modify: `mira-vocabulary/src/{lib.rs,ids/kinds.rs,error/codes/*}`, `mira-core/src/{lib.rs,storage/migrations/mod.rs,storage/repositories/mod.rs}`
- Test: `mira-core/tests/templates_render.rs`, `mira-core/tests/storage_templates.rs`

**Interfaces:**
- Produces (vocabulaire) :
  - `TemplateId` (même macro que `LocalLabelId`, `from_raw`, `as_str`)
  - `MessageTemplate { id: TemplateId, name: String, subject: Option<String>, body: String, created_at: i64, updated_at: i64 }`
  - `ContactInfo { name: Option<String>, first_name: Option<String>, email: Option<String> }`
  - `RenderContext { contact: Option<ContactInfo>, account_name: Option<String>, account_email: Option<String>, subject: Option<String>, date: Option<String>, time: Option<String> }` (`Default`)
  - `RenderedTemplate { subject: Option<String>, body: String, unresolved: Vec<String> }`
  - Tous `Debug, Clone, PartialEq, Eq, Serialize, Deserialize`.
- Produces (moteur) :
  - `templates::render(text: &str, context: &RenderContext) -> (String, Vec<String>)`
  - `templates::render_template(template: &MessageTemplate, context: &RenderContext) -> RenderedTemplate` (l'objet et le corps partagent la liste `unresolved`, sans doublon)
  - `Tx::insert_template(&self, &MessageTemplate)`, `update_template(&self, &TemplateId, name: &str, subject: Option<&str>, body: &str, now: i64)`, `delete_template(&self, &TemplateId)`, `get_template(&self, &TemplateId) -> Result<Option<MessageTemplate>, MiraError>`, `list_templates(&self) -> Result<Vec<MessageTemplate>, MiraError>` (tri par nom sans casse, puis id), `count_templates(&self) -> Result<usize, MiraError>`
  - `Tx::template_idempotency_entry(&self, &IdempotencyKey)` et `record_template_idempotency(&self, &IdempotencyKey, &TemplateIdempotencyEntry)` (même forme que les libellés)
  - Erreurs : `invalid_input.template_name_empty`, `invalid_input.template_name_taken`, `invalid_input.template_too_long`, `not_found.template`.

- [ ] **Step 1: Écrire les tests de rendu (`templates_render.rs`)** — appels directs à `mira_core::templates::render`. Noms et assertions :
  - `each_variable_is_replaced` : un contexte complet (`contact.nom = "Durand"`, `contact.prénom = "Léa"`, `contact.email = "lea@x.fr"`, `date = "25/10/2026"`, `heure = "14:30"`, `moi.nom = "Noah"`, `moi.email = "n@x.fr"`, `objet = "Devis"`) ; chaque `{{variable}}` du texte devient la valeur, `unresolved` est vide.
  - `spaces_inside_braces_are_tolerated` : `{{ date }}` → `25/10/2026`.
  - `unknown_variable_stays_visible_and_is_listed` : `{{truc}}` reste, `unresolved == ["truc"]`.
  - `missing_or_empty_value_stays_visible` : contexte sans contact et `moi.nom = Some("")` → `{{contact.nom}}` et `{{moi.nom}}` restent, listés.
  - `unresolved_is_deduplicated_in_order` : `{{b}} {{a}} {{b}}` → `["b", "a"]`.
  - `value_is_not_resubstituted` : `contact.nom = "{{date}}"` → le texte contient `{{date}}` littéral, `unresolved` vide.
  - `unclosed_braces_do_not_fail` : `"Bonjour {{contact.nom"` revient inchangé.

- [ ] **Step 2: Lancer pour voir l'échec** — `cd engine && cargo test -p mira-core --test templates_render` ; attendu : erreur de compilation (module `templates` absent).

- [ ] **Step 3: Implémenter le vocabulaire et le rendu** — types ci-dessus. `variables.rs` : `enum TemplateVariable { ContactName, ContactFirstName, ContactEmail, Date, Time, MyName, MyEmail, Subject }`, `fn parse(name: &str) -> Option<TemplateVariable>` et `fn value(&self, context: &RenderContext) -> Option<&str>` (un `match` exhaustif ; une valeur vide donne `None`). `render.rs` : balayage unique de gauche à droite repérant `{{`…`}}`, nom nettoyé par `trim` ; un `{{` sans `}}` est recopié tel quel.

- [ ] **Step 4: Relancer** — `cargo test -p mira-core --test templates_render` ; attendu : 7 tests PASS.

- [ ] **Step 5: Écrire les tests de stockage (`storage_templates.rs`)**, en s'inspirant de `storage_events.rs` pour ouvrir une base de test :
  - `insert_then_list_sorted_case_insensitive`
  - `name_is_unique_ignoring_case_and_trim` (Review Focus 1) → `invalid_input.template_name_taken`
  - `empty_name_is_refused` → `template_name_empty`
  - `too_long_fields_are_refused` (Review Focus 2) → `template_too_long`, `count_templates() == 0`
  - `update_keeps_id_and_changes_updated_at`
  - `delete_unknown_is_not_found` → `not_found.template`
  - `migration_applies_on_existing_database` : base à `user_version = 11`, ouverture, `schema_version() == 12` et table présente.

- [ ] **Step 6: Implémenter migration et dépôt** — `v012_message_templates.sql` : `message_templates (id TEXT PRIMARY KEY, name TEXT NOT NULL, name_key TEXT NOT NULL UNIQUE, subject TEXT, body TEXT NOT NULL, created_at INTEGER NOT NULL, updated_at INTEGER NOT NULL)` et `template_idempotency (key TEXT PRIMARY KEY, operation TEXT NOT NULL, fingerprint TEXT NOT NULL, result TEXT NOT NULL)`. Ajouter `include_str!("v012_message_templates.sql")` à `MIGRATIONS` ; corriger les tests existants qui figent `schema_version()` à 11 (`grep -rn "schema_version" engine/crates`), chemin d'import seulement ou valeur attendue 12. Dépôt : mêmes conventions que `local_labels` (`rows.rs` avec `COLUMNS`, `name_key`, erreurs ; `read.rs` ; `write.rs` ; `idempotency.rs`). Les bornes vivent dans `rows.rs` (constantes `MAX_NAME`, `MAX_SUBJECT`, `MAX_BODY`).

- [ ] **Step 7: Relancer** — `cargo test -p mira-core --test storage_templates --test templates_render` ; `cargo clippy -p mira-core -p mira-vocabulary --all-targets -- -D warnings` ; `tools/check-norms` ; attendu : tout vert.

- [ ] **Step 8: Commit**

```bash
git add engine docs
git commit -m "feat(modèles): entité, migration v012, dépôt et rendu des variables"
```

---

### Task 2: Domaine d'API, événement et contrat

**Files:**
- Create: `mira-core/src/api/contract/template_{requests,responses}.rs`, `mira-core/src/api/handlers/template/{mod,list,get,create,update,delete,render,checks,idempotency,writing}.rs`
- Modify: `api/contract/{mod,request,response}.rs`, `api/router.rs`, `api/handlers/mod.rs`, `mira-vocabulary/src/events/{kind,category}.rs`
- Test: `mira-core/tests/api_templates.rs`, `mira-core/tests/contract_examples/template_domain.rs` (+ branchement dans `snapshot.rs`, `mod.rs`), `mira-core/tests/contract_snapshot_templates.rs`

**Interfaces:**
- Consumes: Task 1 (types, `Tx::*_template`, `templates::render_template`).
- Produces :
  - `TemplateRequest` (`#[serde(tag = "op", rename_all = "snake_case")]`) : `ListTemplates`, `GetTemplate { template: TemplateId }`, `CreateTemplate { name: String, subject: Option<String>, body: String, idempotency_key: IdempotencyKey }`, `UpdateTemplate { template, name, subject, body }`, `DeleteTemplate { template }`, `RenderTemplate { template, context: RenderContext }`
  - `TemplateResponse` (`tag = "type", content = "data"`) : `Template(MessageTemplate)`, `Templates(Vec<MessageTemplate>)`, `Rendered(RenderedTemplate)`, `Done`
  - `Request::Template(TemplateRequest)`, `Response::Template(TemplateResponse)`
  - `EventKind::TemplatesChanged` (catégorie `EventCategory::Template`, sans compte), émis après chaque écriture réussie, jamais après lecture ni rejeu d'une clé déjà jouée.

- [ ] **Step 1: Écrire `api_templates.rs`** sur le modèle de `api_local_labels.rs` (même `Fixture`, helpers dans un module `support_template_api` à créer, calqué sur `support_label_api`) :
  - `create_then_list_returns_templates_sorted`
  - `create_with_same_key_replays_without_duplicate_or_event` (invariant 18)
  - `same_key_other_payload_is_idempotency_conflict`
  - `update_changes_fields_and_emits_one_event`
  - `update_to_taken_name_is_refused`
  - `delete_then_get_is_not_found` (`not_found.template`)
  - `render_replaces_variables_and_reports_unresolved` : modèle `Bonjour {{contact.prénom}}, le {{date}}`, contexte sans `contact` → `unresolved == ["contact.prénom"]`, corps `Bonjour {{contact.prénom}}, le 25/10/2026`
  - `render_unknown_template_is_not_found`
  - `reads_emit_no_event` : liste, lecture et rendu n'émettent rien ; `changed_count` égal au nombre d'écritures.
  - `template_content_never_appears_in_errors_or_events` : un corps `SECRET-MARK-42` n'apparaît dans aucun message d'erreur ni payload d'événement.

- [ ] **Step 2: Lancer pour voir l'échec** — `cd engine && cargo test -p mira-core --test api_templates` ; attendu : erreur de compilation.

- [ ] **Step 3: Implémenter contrat, handlers et événement** — un fichier par opération, `route` en `match` exhaustif (comme `local_label/mod.rs`). `create.rs` suit `local_label/create.rs` (`TemplateCommand` copié de `LabelCommand` dans `idempotency.rs`, `announce` dans `writing.rs`). `update.rs` et `delete.rs` : mêmes contrôles que `rename.rs` / `delete.rs`. Ajouter `TemplatesChanged` aux trois `match` de `kind.rs` (`category`, `account`) et `Template` à `EventCategory`.

- [ ] **Step 4: Instantanés de contrat** — créer `template_domain.rs` (exemples de chaque requête et réponse, comme `local_label_domain.rs`) et le brancher dans `snapshot.rs` ; régénérer : `MIRA_UPDATE_SNAPSHOT=1 cargo test -p mira-core --test contract_snapshot_requests_and_responses --test contract_snapshot_events` puis **relire le diff** de `tests/snapshots/contract_v1.json` et `events_v1.json` (seuls des ajouts attendus). `contract_snapshot_templates.rs` vérifie la forme des 6 requêtes et 4 réponses.

- [ ] **Step 5: Relancer** — `cargo test -p mira-core` (toutes les suites touchées par le contrat et les événements) ; `cargo test` dans `bridge/` (le pont ne doit rien exiger de plus) ; `cargo clippy --all-targets -- -D warnings` ; `tools/check-norms`, `tools/check-architecture` ; attendu : vert.

- [ ] **Step 6: Consigner la décision** dans `docs/DECISIONS-AGENT.md` (contexte / choix / raison / retour arrière : points (a) à (c) des contraintes globales).

- [ ] **Step 7: Commit**

```bash
git add engine docs
git commit -m "feat(modèles): API des modèles de messages, événement TemplatesChanged et contrat"
```

---

### Task 3: Couche Flutter et écran de gestion

**Files:**
- Create: `app/lib/engine/template_admin.dart`, `app/lib/state/templates.dart`, `app/lib/ui/templates_settings.dart`
- Modify: `app/lib/engine/{event_routes,engine_mail_source,mail_source}.dart`, `app/lib/state/store.dart`, `app/lib/ui/settings_panel.dart`, `app/lib/i18n/` (textes et `engine_errors.dart`)
- Test: `app/test/template_admin_test.dart`, `app/test/templates_state_test.dart`, `app/test/templates_settings_test.dart`, `app/test/event_routes_test.dart` (existant, vérifie la table)

**Interfaces:**
- Consumes: Task 2 (contrat JSON : domaine `template`, ops en snake_case ; événement `templates_changed`).
- Produces :
  - `class EngineTemplate { String id, name; String? subject; String body; fromJson(Map<String, dynamic>) }`
  - `class RenderedTemplate { String? subject; String body; List<String> unresolved }`
  - `class TemplateAdmin(EngineClient client)` : `Future<List<EngineTemplate>> list()`, `create(String name, String? subject, String body)`, `update(String id, String name, String? subject, String body)`, `delete(String id)`, `Future<RenderedTemplate> render(String id, Map<String, dynamic> context)`
  - `class Templates extends ChangeNotifier` (liste en mémoire, `reload()`), exposé par `MailStore`
  - `EventEffect.templates`, route `'templates_changed': {EventEffect.templates}`

- [ ] **Step 1: Écrire les tests** (même style que `local_label_admin_test.dart` : faux `EngineClient`) :
  - `template_admin_test.dart` : `list` décode ; `create` envoie `domain: template`, `op: create_template` avec une clé `idempotency_key` non vide et différente à chaque appel ; `render` décode `unresolved`.
  - `templates_state_test.dart` : l'événement `templates_changed` déclenche un rechargement ; une erreur du moteur est exposée sans vider la liste.
  - `templates_settings_test.dart` : créer, renommer, supprimer (avec confirmation) ; nom en double → message traduit ; barre de variables : toucher `contact.nom` insère `{{contact.nom}}` au curseur.
  - `event_routes_test.dart` : la table nomme `templates_changed` (le test existant compare à `events_v1.json`).

- [ ] **Step 2: Lancer pour voir l'échec** — `cd app && flutter test test/template_admin_test.dart` ; attendu : fichier source absent.

- [ ] **Step 3: Implémenter** — `template_admin.dart` calqué sur `local_label_admin.dart` (`_send`, clé `ui-template-create-…`). Brancher `EventEffect.templates` dans `event_routes.dart` et le `switch` de `engine_mail_source.dart` ; exposer `templateAdmin` dans `MailSource`. `templates_settings.dart` : liste, éditeur (nom, objet, corps), barre de variables (les 8), suppression avec confirmation, erreurs via `i18n/engine_errors.dart` (`invalid_input.template_name_empty`, `template_name_taken`, `template_too_long`, `not_found.template`). Ajouter l'entrée dans `settings_panel.dart` (nouvel onglet, garde `s.source?.templateAdmin != null` comme `labelSettingsAdmin`). Textes dans `i18n` ; `i18n_coverage_test` doit rester vert.

- [ ] **Step 4: Relancer** — `flutter test test/template_admin_test.dart test/templates_state_test.dart test/templates_settings_test.dart test/event_routes_test.dart test/events_contract_test.dart test/i18n_coverage_test.dart` ; `flutter analyze` ; attendu : vert.

- [ ] **Step 5: Commit**

```bash
git add app
git commit -m "feat(app): gestion des modèles de messages (liste, édition, variables)"
```

---

### Task 4: Insertion dans l'éditeur (bouton et raccourci `/`)

**Files:**
- Create: `app/lib/ui/template_picker.dart`, `app/lib/state/template_context.dart` (construction du `RenderContext`)
- Modify: `app/lib/ui/composer.dart` (barre de formatage `_FmtBar`, gestion de la saisie), `app/lib/state/rich_controller.dart` si la surbrillance l'exige
- Test: `app/test/template_picker_test.dart`, `app/test/template_context_test.dart`, `app/test/composer_templates_test.dart`

**Interfaces:**
- Consumes: Task 3 (`TemplateAdmin.render`, `Templates`).
- Produces :
  - `Map<String, dynamic> buildTemplateContext(ComposeState c, ContactBook contacts, DateTime now)` : destinataire principal = premier de `c.to` (nom, prénom = premier mot du nom, email) ; `moi.*` = identité d'envoi `c.from` ; `objet` = `c.subject.text` ; `date` `JJ/MM/AAAA` et `heure` `HH:MM` formatées par l'app. Une valeur vide n'est pas envoyée.
  - `Future<EngineTemplate?> showTemplatePicker(BuildContext, List<EngineTemplate>)` : recherche par nom, Échap ferme sans résultat.
  - `bool slashOpensPicker(String text, int cursor)` : vrai si le caractère avant le curseur est `/` précédé d'un début de ligne ou d'un espace.

- [ ] **Step 1: Écrire les tests** :
  - `template_context_test.dart` : « Léa Durand <lea@x.fr> » donne `contact.prénom = "Léa"` ; aucun destinataire → pas de clé `contact` ; date et heure formatées pour une `DateTime` fixe.
  - `template_picker_test.dart` : la saisie filtre la liste ; Entrée valide ; Échap ferme sans valeur.
  - `composer_templates_test.dart` :
    - `slashOpensPicker("bonjour /", 9)` vrai ; `slashOpensPicker("http://", 7)` faux ; début de ligne vrai.
    - Le bouton « Modèle » ouvre le sélecteur ; choisir un modèle insère le corps rendu à la position du curseur et remplace l'éventuel `/` déclencheur ; l'objet du modèle remplit le champ objet **seulement s'il est vide**.
    - Variables non résolues : restent en clair, le message « N variables à compléter » apparaît.
    - Modèle supprimé entre l'ouverture du sélecteur et l'insertion (Review Focus 6) : l'erreur traduite s'affiche, le corps n'est pas modifié, la liste est rechargée.

- [ ] **Step 2: Lancer pour voir l'échec** — `cd app && flutter test test/template_context_test.dart` ; attendu : fichier source absent.

- [ ] **Step 3: Implémenter** — `template_context.dart` et `template_picker.dart` en fichiers séparés ; dans `composer.dart`, seulement le branchement (bouton dans `_FmtBar`, appel de `slashOpensPicker` depuis `_onBody`), la logique reste dans les nouveaux fichiers. Surbrillance des variables non résolues via le contrôleur riche existant, ou à défaut une ligne d'état sous l'éditeur si `RichController` ne s'y prête pas (consigner le choix dans `DECISIONS-AGENT.md`).

- [ ] **Step 4: Relancer** — `flutter test test/template_context_test.dart test/template_picker_test.dart test/composer_templates_test.dart` puis la suite complète `flutter test` (le composeur est partagé) ; `flutter analyze` ; attendu : vert.

- [ ] **Step 5: Commit**

```bash
git add app docs
git commit -m "feat(app): insertion des modèles dans l'éditeur (bouton et raccourci /)"
```

---

## Fin de parcours

Après la tâche 4 : demander à l'utilisateur l'accord pour lancer `./ci.sh`, puis un relecteur indépendant sur la branche entière avant fusion dans `dev`.

## Auto-relecture

- **Couverture du spec :** variables et comportement « visible » (T1), stockage local global et idempotence (T1, T2), contrat, erreurs typées et événement (T2), écran de gestion (T3), bouton, `/`, objet, surbrillance (T4), hors périmètre respecté. Le test « aucun contenu dans journaux ni erreurs » est dans T2.
- **Cohérence des types :** `RenderContext`, `RenderedTemplate`, `TemplateId`, `TemplateRequest/Response` et `EventKind::TemplatesChanged` sont définis en T1/T2 et réutilisés tels quels en T3/T4 (côté Dart : `templates_changed`, `render`, `unresolved`).
- **Écart au spec, assumé :** le spec parle d'un contexte avec fuseau ; le plan fait fournir date et heure déjà formatées par l'app (décision (a)), ce qui garde le moteur sans notion de locale.
