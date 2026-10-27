# Export et import des règles en YAML — design

Date : 2026-10-27. Statut : à relire. Branche : `feat/yaml-import-export`.

## Objectif

Permettre de sauvegarder toutes ses règles dans un fichier `.yaml` et de les restaurer, y compris sur une autre machine. Priorité 1 : sauvegarde et transfert (aller-retour fidèle). Priorité 2 : fichier lisible et modifiable à la main, avec des erreurs précises.

Ce chantier est le premier de trois, dans l'ordre : règles, libellés locaux, modèles de messages. Le schéma d'enveloppe et le mécanisme d'import décrits ici sont conçus pour être réutilisés. Chaque entité aura sa propre spec et son propre plan.

## Décisions de cadrage

- **Approche A** : le moteur convertit du **texte YAML**. Flutter lit et écrit le fichier (`file_picker`) et ne fait qu'appeler. Pas d'I/O fichier dans le moteur. Pas de nouvelle fonction FFI : le pont transporte l'enveloppe existante.
- **Références portables** : un identifiant (`MailboxId`, `LocalLabelId`, `TemplateId`, comptes du `scope`) est propre à une installation. Le fichier écrit des références en clair, résolues à l'import (voir Format).
- **Références non résolues** : la règle fautive est **refusée**, les autres sont importées, un rapport détaille chaque refus. Aucune création automatique de libellé ou de modèle dans cette version.
- **Doublons** (règle de même identifiant déjà présente) : **ignorée** par défaut, statut « déjà présente ». Option `replace_existing` : la règle existante est remplacée par celle du fichier.
- **Périmètre de l'export** : toutes les règles, dans l'ordre d'évaluation, avec leur état activé ou désactivé.
- Toute la logique vit dans le moteur Rust (conversion, résolution, rapport). Le cœur ne nomme aucun provider.

## Format

Document YAML, un seul par fichier.

```yaml
mira_export:
  kind: rules
  version: 1
rules:
  - id: "…"
    name: "Factures"
    enabled: true
    scope: …            # comptes désignés par leur adresse
    alternatives: …     # mêmes groupes de conditions que le modèle Rule
    actions:
      - move_to: { account: "moi@exemple.fr", mailbox: "Archives/Factures" }
      - add_local_label: { name: "Compta" }
      - reply_with_template: { name: "Accusé de réception" }
```

- `kind` et `version` sont vérifiés en premier. Un `kind` autre que `rules` ou une version inconnue refuse le fichier entier.
- Références : compte → son adresse ; boîte → adresse du compte + chemin de la boîte ; libellé local → son nom (insensible à la casse et aux espaces de bord, comme `name_key`) ; modèle → son nom (idem).
- Les conditions et le reste de la règle gardent la représentation du modèle `Rule` existant (`schema_version` conservé) ; seules les références changent.
- Les champs inconnus dans une règle sont **refusés**, pour qu'une faute de frappe ne passe pas inaperçue.

## Contrat d'API

Deux variantes ajoutées à `RuleRequest`, deux à `RuleResponse` :

- `ExportRules` → `RulesExport { yaml: String }`.
- `ImportRules { yaml: String, replace_existing: bool, idempotency_key: IdempotencyKey }` → `RulesImport(ImportReport)`.
- `ImportReport { entries: Vec<ImportEntry> }`, avec `ImportEntry { position: u32, name: String, outcome: ImportOutcome }` et `ImportOutcome` ∈ `Imported`, `Replaced`, `AlreadyPresent`, `Refused { reason }`.
- `reason` est un code stable : `unknown_account`, `unknown_mailbox`, `unknown_label`, `unknown_template`, `invalid_rule` (porte l'erreur du validateur de règles existant).
- Erreurs de fichier entier : `invalid_input.yaml_syntax` (avec ligne et colonne, sans extrait du contenu), `invalid_input.export_version_unsupported`, `invalid_input.export_kind_mismatch`, `invalid_input.export_too_large`.
- `ImportRules` est idempotent par clé : même clé, même rapport rejoué (stocké dans une colonne `result` ajoutée à `rule_idempotency`, migration v014), aucune écriture ni événement ; même clé avec un autre contenu → conflit d'idempotence.
- Événement `RuleChanged` (existant) émis une fois par règle écrite (importée ou remplacée), jamais après une lecture, un rejeu ou un import sans effet.

## Comportement de l'import

1. Contrôle de taille puis analyse YAML ; contrôle de `kind` et `version`.
2. Dans **une seule transaction**, règle par règle : résolution des références, traitement du doublon (ignorer ou remplacer), validation par le validateur existant, écriture.
3. Une règle refusée n'empêche pas les suivantes. En cas d'échec de la base, **rien n'est écrit** (atomicité).
4. L'ordre du fichier est conservé : les règles ajoutées sont placées à la suite des règles existantes dans l'ordre du fichier ; une règle remplacée garde sa position.
5. Le rapport couvre toutes les règles du fichier.

## Bornes de sécurité

Fichier ≤ 2 Mo, ≤ 500 règles, refus typé `invalid_input.export_too_large` au-delà. Valeurs à consigner dans `docs/DECISIONS-AGENT.md`.

## Secrets et journaux

Aucun contenu de règle (noms, motifs de conditions, adresses) dans les journaux, les messages d'erreur ni les payloads d'événements. Les positions et les codes de raison suffisent.

## Dépendance YAML

`serde_yaml` est archivée (publiée `0.9.34+deprecated`) et `serde_yml` est signalée non maintenue. Choix retenu : **`yaml_serde`** (maintenue par l'organisation YAML, version 0.10.x au moment de l'écriture), à valider au plan (compilation avec la toolchain 1.99.0, absence d'`unsafe` dans notre code). Repli : `serde_norway`. Le choix est consigné dans `DECISIONS-AGENT.md` avec retour arrière (la conversion vit dans un seul module, remplaçable).

## Organisation du code

- `mira-core/src/portability/` (nouveau, générique, réutilisable pour libellés et modèles) : enveloppe `mira_export`, contrôle de version et de taille, `ImportReport`.
- `mira-core/src/portability/rules/` : conversion `Rule` ↔ document, résolution des références, import et export.
- `mira-core/src/api/handlers/rule/export.rs` et `import.rs` : un handler par opération, routage par `match` exhaustif.
- `mira-vocabulary` : types du rapport et nouveaux codes d'erreur.
- Normes de codage du projet (fonction ≤ 30 lignes, fichier ≤ 6 fonctions, pas de commentaire dans un corps de fonction).

## Interface Flutter

Dans `rules_settings.dart` (pas de nouvelle fonction FFI) :

- Deux boutons dans l'en-tête : **Exporter** et **Importer**.
- Exporter : `ExportRules`, puis `FilePicker.saveFile` avec un nom du type `mira-regles-2026-10-27.yaml` (mécanisme déjà utilisé dans `store.dart`).
- Importer : `FilePicker.pickFiles` (`.yaml`, `.yml`), lecture du texte, boîte de dialogue avec la case « Remplacer les règles existantes » (décochée), puis `ImportRules`.
- Rapport affiché après l'import : résumé (« 5 importées, 2 déjà présentes, 1 refusée ») et détail des refus avec raison traduite.
- Rafraîchissement de la liste par `RulesChanged`. Textes dans `i18n` ; `i18n_coverage_test` reste vert. Codes d'erreur traduits dans `i18n/engine_errors.dart`.
- Sur Android, le sélecteur du système fournit le contenu sans chemin direct, ce qui convient à l'approche A.

## Tests

- **Moteur** : aller-retour fidèle (export puis import sur une base vierge, règles identiques, ordre compris) ; chaque cause de refus ; doublons dans les deux modes ; atomicité ; idempotence (invariant 18) ; absence de contenu dans erreurs et événements ; bornes de taille ; fichier invalide (syntaxe avec ligne et colonne, version, `kind`, champ inconnu).
- **Contrat** : instantanés des nouvelles requêtes, réponses et codes d'erreur.
- **Flutter** : requêtes de `rule_admin`, boîte de dialogue (case à cocher, rapport avec refus), traduction des codes.

## Hors périmètre

Export d'une sélection de règles ; création automatique des libellés ou modèles manquants ; import des libellés locaux et des modèles (cycles suivants) ; synchronisation ou partage entre personnes (références propres à chaque compte).

## Cas à relire en priorité

1. Une règle référençant un libellé supprimé ou renommé entre export et import : refusée, `unknown_label`.
2. Deux règles du fichier portant le même `id` : la seconde est refusée comme doublon interne, pas d'écriture double.
3. Remplacement d'une règle par une version dont une référence ne se résout pas : la règle existante reste intacte (refus, pas de suppression préalable).
4. Fichier modifié à la main avec un nom de libellé en casse différente : résolu comme `name_key`.
5. Import rejoué après un crash en cours de transaction : rien d'écrit, aucun état partiel.
