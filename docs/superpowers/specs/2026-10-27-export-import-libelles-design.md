# Export et import des libellés locaux en YAML — design

Date : 2026-10-27. Statut : à relire. Branche : `feat/yaml-import-export` (suite du chantier des règles, AG-81).

## Objectif

Sauvegarder les libellés locaux dans un fichier `.yaml` et les restaurer, y compris sur une autre machine. Priorité 1 : sauvegarde et transfert (aller-retour fidèle). Priorité 2 : fichier lisible et modifiable à la main. Deuxième des trois chantiers (règles, **libellés locaux**, modèles de messages).

Les libellés s'importent **avant** les règles : une règle qui vise un libellé par son nom n'est plus refusée pour `unknown_label` une fois le libellé présent.

## Décisions de cadrage (héritées d'AG-81, sauf mention)

- Le moteur convertit du **texte** YAML ; l'app lit et écrit le fichier. Même enveloppe `mira_export` (`kind: local_labels`, `version: 1`), mêmes bornes de fichier (2 Mo), mêmes erreurs de fichier entier, ancres YAML refusées (`portability::envelope`, inchangé).
- **Seulement les libellés** (nom et couleur), pas leur pose sur les conversations : les identifiants de conversation sont propres à une installation. La pose pourra s'ajouter plus tard sans changer le format.
- Un libellé est identifié par son **nom** (unique, casse et espaces de bord ignorés, comme `name_key`). Pas d'identifiant dans le fichier.
- **Doublon** (nom déjà pris) : ignoré par défaut, « déjà présent ». Option `replace_existing` : le libellé existant prend le nom (casse comprise) et la couleur du fichier. **Un remplacement qui ne change rien est rapporté « déjà présent »**, sans écriture ni événement (leçon de la revue des règles).
- Tous les libellés, dans l'ordre de `ListLocalLabels` (nom sans casse). Pas de sélection.
- Un libellé refusé n'empêche pas les suivants ; le rapport dit pourquoi. Une transaction ; atomicité.

## Format

```yaml
mira_export:
  kind: local_labels
  version: 1
labels:
  - name: Compta
    color: "#ff1e90ff"
  - name: Voyages
    color: "#ff2e8b57"
```

- `color` : texte `#AARRGGBB` à l'export. À l'import, `#RRGGBB` (opaque) et `#AARRGGBB` sont acceptés, hexadécimal insensible à la casse ; l'entier ARGB de `LocalLabel.color` en est la valeur exacte.
- `color` absent : couleur par défaut choisie comme à la création (`default_color`). Champs inconnus : libellé refusé (`invalid_label`).
- L'ordre du fichier est celui de l'export ; il n'a pas de sens à l'import (les libellés n'ont pas d'ordre propre).

## Contrat d'API

Deux variantes ajoutées à `LocalLabelRequest`, deux à `LocalLabelResponse` (l'opération existante `ImportLocalLabels`, reprise des libellés de l'ancienne version de l'app, reste inchangée) :

- `ExportLocalLabelsFile` → `LabelsFileExport { yaml: String }`.
- `ImportLocalLabelsFile { yaml: String, replace_existing: bool, idempotency_key: IdempotencyKey }` → `LabelsFileImport(LabelImportReport)`.
- `LabelImportReport { entries: Vec<LabelImportEntry> }`, `LabelImportEntry { position: u32, name: String, outcome: LabelImportOutcome }`, `LabelImportOutcome` ∈ `Imported`, `Replaced`, `AlreadyPresent`, `Refused { reason: LabelImportRefusal }`, `LabelImportRefusal` ∈ `InvalidName`, `InvalidColor`, `InvalidLabel`, `DuplicateInFile`, `TooManyLabels`. Positions à partir de 1 ; le nom du rapport est tronqué à 64 caractères (leçon de la revue).
- Erreurs de fichier entier : celles de l'enveloppe, inchangées.
- Rejeu par clé : le rapport est mémorisé par le mécanisme existant des libellés (`LabelCommand::record`, résultat sérialisé) : **aucune migration**. Même clé et autre contenu ou autre option → `invalid_input.idempotency_conflict`. Empreinte : `(replace_existing, SHA-256 du texte)`.
- Événement `LocalLabelsChanged` (existant), une seule fois après une importation qui a écrit quelque chose ; jamais après lecture, rejeu ou import sans effet.

## Comportement de l'import

1. Enveloppe : taille, ancres, syntaxe, `kind`, version, nombre d'éléments.
2. Dans une transaction, libellé par libellé, dans l'ordre du fichier : lecture stricte (nom, couleur ; champ inconnu refusé) ; nom contrôlé par `clean_name` (non vide, ≤ 64) ; doublon interne au fichier (même clé de nom) refusé ; doublon existant ignoré ou remplacé ; plafond de 200 libellés (`MAX_LABELS`) : au-delà, les nouveaux sont refusés `TooManyLabels`.
3. Aucune écriture si la base échoue en cours de route.
4. Le rapport couvre tous les libellés du fichier.

## Secrets et journaux

Aucun nom de libellé dans les journaux, erreurs ni payloads d'événements (`LocalLabelsChanged` n'en porte aucun). Les noms n'apparaissent que dans le rapport rendu à l'appelant.

## Organisation du code

- `mira-core/src/portability/labels/{mod,to_portable,from_portable,color}.rs` : forme portable pure d'un libellé (conversion couleur, lecture stricte).
- `mira-core/src/api/handlers/local_label/{export_file,import_file,import_file_one}.rs` : un handler par opération.
- `mira-vocabulary` : types du rapport (`LabelImport*`).
- Normes de codage du projet (fonction ≤ 30 lignes, fichier ≤ 6 fonctions, pas de commentaire dans un corps de fonction).

## Interface Flutter

- `labels_settings.dart` : deux boutons **Exporter** et **Importer** ; export dans `mira-libelles-AAAA-MM-JJ.yaml`.
- La boîte d'options et le rapport des règles deviennent **génériques** (titre, note, libellé de la case, raisons de refus fournis par l'appelant) et sont partagés ; l'entrée de rapport Dart (`position`, `name`, `outcome`, `reason`) est commune aux règles et aux libellés.
- Gardes de l'app (leçons de la revue) : boutons désactivés pendant une opération, lecture du fichier bornée y compris quand la taille est inconnue.
- Textes dans `i18n` (français + anglais) ; codes de refus traduits.

## Tests

- **Moteur** : aller-retour fidèle (noms, couleurs) ; chaque cause de refus ; doublons dans les deux modes, dont « remplacement identique = déjà présent, aucun événement » ; doublon interne ; plafond de 200 ; atomicité (déclencheur SQL) ; idempotence (invariant 18) et conflits ; confidentialité (marqueur secret) ; couleur `#RRGGBB`, `#AARRGGBB`, majuscules/minuscules, invalides ; fichier invalide, version, `kind`.
- **Contrat** : instantanés des nouvelles requêtes, réponses et codes.
- **Flutter** : `LabelAdmin` (requêtes, rapport), boîte d'import générique, boutons des libellés, erreurs de fichier, garde « occupé ».

## Hors périmètre

Pose des libellés sur les conversations ; export d'une sélection ; couleurs nommées (`red`…) ; export/import des modèles (chantier suivant).

## Cas à relire en priorité

1. Réimporter sa propre sauvegarde avec « remplacer » : tout est « déjà présent », aucune écriture, aucun événement.
2. Fichier contenant `compta` et `Compta` : le second est refusé (`DuplicateInFile`), un seul libellé est écrit.
3. 190 libellés existants, fichier de 20 nouveaux : 10 importés, 10 refusés `TooManyLabels`, rapport exact.
4. Couleur `#FFF` (forme courte) ou `rouge` : libellé refusé `InvalidColor`, les autres sont importés.
5. Nom de 1,9 Mo dans une entrée refusée : le rapport le tronque, rien d'énorme n'est mémorisé.
