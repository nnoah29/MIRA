# Libellés locaux dans le moteur — plan d'implémentation

> **Pour les agents :** sous-compétence requise : `superpowers:executing-plans` (exécution directe dans la session, sans sous-agent, choix de l'utilisateur). Les étapes utilisent des cases `- [ ]`.

**Objectif :** déplacer les libellés locaux de l'app vers le moteur, par conversation, et les rendre utilisables par les règles.

**Architecture :** types dans `mira-vocabulary` ; tables v008 et dépôt dans `mira-core/storage` ; domaine d'API `local_label` ; extension de `rules/` (condition, actions, évaluation, validation, catalogue, application) ; côté app, un `LocalLabelAdmin` et une bascule moteur/préférences dans `MailStore`.

**Spec :** `docs/specs/R2-libelles-locaux.md` (et `R1-regles.md` pour les règles).

## Contraintes globales

- Normes `tools/check-norms` : fonction ≤ 30 lignes ; fichier ≤ 6 fonctions avec corps (tests compris) ; aucun commentaire dans un corps de fonction ; une seule ligne `///` française avant chaque `fn` ; `//!` de 1–3 lignes en tête de fichier ; pas de `unwrap`/`expect`/`panic!` hors tests ; pas de `unsafe` ; pas de `#[rustfmt::skip]` ; interdits : `utils.rs`, `helpers.rs`, `common.rs`, `misc.rs`.
- Le cœur ne nomme aucun provider ; `mira-vocabulary` ne dépend que de `serde` et `thiserror`.
- Pas de `./ci.sh` (accord utilisateur requis) : `cargo test -p <crate> --test <fichier>`, `clippy -D warnings`, `fmt` ciblés ; le pont se compile depuis `bridge/` (jamais `--manifest-path` depuis la racine).
- Commits au nom de l'utilisateur, sans trailer d'attribution ; ne pas toucher `app/lib/main.dart` ni `dart format` sur tout `lib/` (reformate `main.dart`).
- Pose rattachée à `(compte, clé du fil)`, jamais à l'identifiant du fil, sans clé étrangère vers `threads`.
- Conventions de l'app : textes français comme clés `t('…')`, traductions dans `app/lib/i18n/en.dart` et `app/tool/en.py`.

## Points d'attention (Review Focus)

1. Recalcul des fils (`recompute_threads`) et rejet du miroir : les poses subsistent.
2. Libellé supprimé alors qu'une règle l'utilise : la règle devient invalide, jamais exécutée.
3. Reprise de l'app : un échec garde la copie locale ; un rejeu ne duplique rien ; une clé de conversation inconnue est ignorée et comptée.
4. Nom vide, trop long ou doublon (sans casse) : refus sans écriture ; aucun contenu saisi dans les erreurs.
5. Une règle qui pose un libellé est suivie d'une règle qui le teste : la seconde voit l'effet de la première.

---

### Task 1: Types, stockage et dépôt

**Files:**
- Create: `engine/crates/mira-vocabulary/src/domain/local_label.rs` ; `engine/crates/mira-core/src/storage/migrations/v008_local_labels.sql` ; `engine/crates/mira-core/src/storage/repositories/local_labels/{mod,rows,read,write,assignments}.rs`
- Modify: `mira-vocabulary/src/{ids/kinds.rs,domain/mod.rs,lib.rs}`, `storage/migrations/mod.rs`, `storage/repositories/mod.rs`
- Test: `engine/crates/mira-core/tests/storage_local_labels.rs` (+ `tests/support_local_labels/` si besoin)

**Interfaces:**
- Produit : `LocalLabelId`, `LocalLabel { id, name, color: u32 }`, `LocalLabelAssignment { thread: ThreadId, labels: Vec<LocalLabelId> }` ; méthodes de `Tx` : `insert_local_label(&LocalLabel)`, `rename_local_label(&LocalLabelId, &str)`, `recolor_local_label(&LocalLabelId, u32)`, `delete_local_label(&LocalLabelId)`, `get_local_label`, `list_local_labels() -> Vec<LocalLabel>`, `set_thread_label(&LocalLabelId, &AccountId, &str /*root_key*/, bool)`, `labels_of_thread(&ThreadId) -> Vec<LocalLabelId>`, `list_label_assignments() -> Vec<LocalLabelAssignment>` (fils existants seulement), `thread_root_key(&ThreadId) -> Option<(AccountId, String)>`.

- [ ] **Step 1 : tests** — `create_get_list_roundtrip` ; `duplicate_name_ignoring_case_is_refused` ; `rename_and_recolor` ; `delete_label_removes_its_assignments` ; `set_and_unset_thread_label` ; `assignment_on_unknown_thread_is_refused` ; `deleting_account_removes_assignments` ; `assignments_survive_recompute_threads` ; `assignments_survive_mirror_discard_and_resync` ; `list_assignments_only_returns_existing_threads`.
- [ ] **Step 2 :** `cd engine && cargo test -p mira-core --test storage_local_labels` — attendu : échec.
- [ ] **Step 3 :** types, migration v008 (`local_labels`, `thread_local_labels` sans FK vers `threads`, FK vers `local_labels` et `accounts` en cascade), dépôt (méthodes de `Tx`, modèle `repositories/rules/`).
- [ ] **Step 4 :** tests PASS ; `storage_database`, `storage_account_cascade`, `threads_recompute` toujours verts ; clippy + fmt.
- [ ] **Step 5 :** `git commit -m "feat(labels): libellés locaux par conversation, stockage"`

### Task 2: API `local_label`, événements, reprise

**Files:**
- Create: `engine/crates/mira-core/src/api/contract/local_label_requests.rs` ; `api/handlers/local_label/{mod,list,create,rename,recolor,delete,set,assignments,import}.rs`
- Modify: `api/contract/{mod,request,response}.rs`, `api/router.rs`, `api/handlers/mod.rs`, `mira-vocabulary/src/events/{kind,category}.rs` (`LocalLabelsChanged`, catégorie `local_label`)
- Test: `engine/crates/mira-core/tests/api_local_labels.rs`, `api_local_labels_import.rs`, mise à jour des snapshots de contrat

**Interfaces:**
- Consomme : Task 1 ; modèle de `api/handlers/rule/` (écritures, idempotence `rule_idempotency` v007 — réutiliser ou ajouter la table suivante si nécessaire).
- Produit : `LocalLabelRequest::{ListLocalLabels, CreateLocalLabel{name,color,idempotency_key}, RenameLocalLabel{label,name}, RecolorLocalLabel{label,color}, DeleteLocalLabel{label}, SetLocalLabel{label,threads,on,idempotency_key}, ListLocalLabelAssignments, ImportLocalLabels{labels,idempotency_key}}`.

- [ ] **Step 1 : tests** — chaque opération ; plafonds (64 octets, 200 libellés, 1000 fils) ; nom vide/doublon refusés sans écriture ; erreurs sans contenu saisi ; idempotence (même clé ⇒ même résultat, clé réutilisée avec d'autres données ⇒ conflit) ; événement émis après changement, pas au rejeu ; `ImportLocalLabels` : libellés + poses, clé résolue comme fil puis comme message, inconnue ignorée et comptée, rejeu sans doublon ; snapshot de contrat en ajouts seuls.
- [ ] **Step 2 :** `cargo test -p mira-core --test api_local_labels --test api_local_labels_import` — attendu : échec.
- [ ] **Step 3 :** un handler par opération, `match` exhaustif ; mettre à jour les `match` exhaustifs d'`EventKind`/`EventCategory`.
- [ ] **Step 4 :** PASS ; `cargo build --workspace --all-targets` ; tests voisins `events_bus`, `api_rules*`.
- [ ] **Step 5 :** `git commit -m "feat(labels): domaine local_label du contrat d'API"`

### Task 3: Extension des règles

**Files:**
- Modify: `mira-vocabulary/src/domain/rule/{test.rs,action.rs}` (`HasLocalLabel`, `AddLocalLabel`, `RemoveLocalLabel`), `mira-core/src/rules/{context.rs,outcome.rs,evaluate.rs,condition_eval.rs,action_translation.rs}`, `rules/validate/*` (`RuleWorld::local_label_exists`, statut), `rules/apply/*` (application des `label_changes`, événement), `api/handlers/rule/{catalog,preview}.rs`, `rule_context/*` (charge `thread_labels`)
- Test: `tests/rules_eval_labels.rs`, `rules_validate_labels.rs`, `sync_rules_labels.rs`, `api_rules_labels.rs`, snapshots de contrat

**Interfaces:**
- Consomme : Tasks 1 et 2 ; `RuleWorld`, `MessageContext`, `RuleOutcome` de R1.
- Produit : `MessageContext.thread_labels: Vec<LocalLabelId>` ; `RuleOutcome.label_changes: Vec<LabelChange { label: LocalLabelId, on: bool }>` ; `RuleWorld::local_label_exists(&self, &LocalLabelId) -> bool`.

- [ ] **Step 1 : tests** — `has_local_label` vrai/faux/négation ; ajout ; retrait ; la dernière règle l'emporte ; une règle voit l'effet de la précédente (point d'attention 5) ; libellé supprimé ⇒ règle `Invalid` et non exécutée (point 2) ; avec « Tous les comptes » acceptées ; application en synchro : pose sur la conversation, événement, rejeu après crash sans changement ; un nouveau mail d'une conversation déjà libellée est vu comme porteur ; aperçu avec libellés réels ; catalogue complet (test de complétude existant) ; snapshots.
- [ ] **Step 2 :** `cargo test -p mira-core --test rules_eval_labels --test rules_validate_labels --test sync_rules_labels --test api_rules_labels` — attendu : échec.
- [ ] **Step 3 :** implémenter ; routage par `match` exhaustif sans joker ; application directe en base (pas de file d'actions).
- [ ] **Step 4 :** PASS ; non-régression `rules_*`, `api_rules*`, `sync_rules*`, `contract_snapshot_*` ; clippy workspace, fmt, `tools/check-norms`.
- [ ] **Step 5 :** `git commit -m "feat(rules): actions et condition sur les libellés locaux"`

### Task 4: Pont

**Files:** Modify `bridge/tests/rules_bridge.rs` (ou nouveau `bridge/tests/local_labels_bridge.rs` si la limite de 6 fonctions l'exige)

- [ ] **Step 1 : test** — `local_label_create_set_list_through_bridge` : créer un libellé, le poser sur une conversation (après une synchro du testkit ou via `ImportLocalLabels`), lister les poses, supprimer.
- [ ] **Step 2 :** `cd bridge && cargo test --test <fichier>` — attendu : PASS.
- [ ] **Step 3 :** `git commit -m "test(bridge): libellés locaux de bout en bout par le pont"`

### Task 5: App Flutter

**Files:**
- Create: `app/lib/engine/local_label_admin.dart`, `app/test/local_label_admin_test.dart`, `app/test/local_label_migration_test.dart`
- Modify: `app/lib/state/store.dart` (+ `state/local_labels.dart` reste pour le mode sans moteur), `app/lib/ui/local_labels_section.dart`, `app/lib/ui/rule_labels.dart`, `app/lib/ui/rule_params.dart` / `rule_editor.dart` (choix de libellé local), `app/lib/i18n/en.dart`, `app/tool/en.py`, `app/lib/engine/engine_mail_source.dart` (événement `local_labels_changed`)
- Test: `app/test/engine_source_test.dart` (non-régression), tests ci-dessus

**Interfaces:**
- Consomme : domaine `local_label` du moteur (JSON `{"domain":"local_label","request":{"op":…}}`).
- Produit : `LocalLabelAdmin(EngineClient)` : `list()`, `create(name, color)`, `rename`, `recolor`, `delete`, `set(label, threads, on)`, `assignments()`, `import(...)`, clés d'idempotence comme `LabelAdmin._key`.

- [ ] **Step 1 : tests** — admin : requêtes et mappers ; store : en mode moteur, libellés et poses viennent du moteur et se relisent sur `local_labels_changed`, `mail.labels` complété par fil ; sans moteur : comportement actuel ; reprise : réussie (préférences effacées, drapeau posé), échouée (copie gardée, nouvel essai), déjà faite (aucun appel) ; éditeur de règles : libellés locaux proposés pour les trois variantes.
- [ ] **Step 2 :** `cd app && flutter test test/local_label_admin_test.dart test/local_label_migration_test.dart` — attendu : échec.
- [ ] **Step 3 :** implémenter ; ne pas lancer `dart format` sur `lib/` entier.
- [ ] **Step 4 :** PASS ; `flutter test` complet une fois ; `flutter analyze` propre.
- [ ] **Step 5 :** `git commit -m "feat(app): libellés locaux portés par le moteur, reprise des libellés existants"`

### Task 6: Documentation

**Files:** Modify `docs/DECISIONS-AGENT.md` (entrées AG-72 et suivantes, format contexte / choix / raison / retour : pose par clé de fil, limite de fusion, reprise par l'app, application directe sans file d'actions), `docs/ARCHITECTURE-IMPLEMENTEE.md` (migration v008).

- [ ] **Step 1 :** écrire les entrées ; relire le format existant et la numérotation.
- [ ] **Step 2 :** `git commit -m "docs(labels): décisions et migration v008"`

---

## Couverture de la spec

§2 types → T1 ; §3 stockage → T1 ; §4 recalcul/fusion → T1 (tests) et T6 (limite documentée) ; §5 API et événement → T2 ; §6 règles → T3 ; §7 app → T5 ; §8 tests → répartis ; §9 lots → T1+T2 (lot 1), T3 (lot 2), T4 (lot 3), T5 (lot 4).
