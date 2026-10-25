# Modèles de messages — design

Date : 2026-10-25. Statut : à relire.

## Objectif

Permettre à l'utilisateur de créer des textes réutilisables (réponses, rédactions) contenant des variables, et de les insérer dans l'éditeur de message. Les variables sont remplacées à l'insertion.

## Décisions de cadrage

- Les modèles sont **locaux et globaux à l'app** : pas de synchro, pas de rattachement à un compte, SQLite local uniquement.
- Une variable sans valeur, ou inconnue, **reste visible** (`{{contact.nom}}`) : on n'envoie jamais un message avec un trou invisible.
- Toute la logique (stockage, parsing, rendu) vit dans le **moteur Rust**. Flutter affiche et appelle (approche 1, comme les libellés locaux, AG-73 à AG-75).
- Périmètre d'interface : écran de gestion, bouton « Modèle » dans l'éditeur, et raccourci `/` pour chercher un modèle.

## Variables

Syntaxe `{{nom}}`, espaces tolérés (`{{ date }}`).

| Variable | Source |
|---|---|
| `contact.nom` | destinataire principal |
| `contact.prénom` | destinataire principal |
| `contact.email` | destinataire principal |
| `date` | date locale fournie par l'app |
| `heure` | heure locale fournie par l'app |
| `moi.nom` | identité d'envoi |
| `moi.email` | identité d'envoi |
| `objet` | objet courant du message |

Un nom inconnu reste tel quel. Un `{{` non fermé ne fait pas échouer le rendu.

## Moteur

**Entité `MessageTemplate`** (`mira-vocabulary`) : `id`, `name` (unique, non vide), `subject` (optionnel), `body`, `created_at`, `updated_at`.

**Stockage** : migration `v012_message_templates.sql`, dépôt dans `storage/repositories`, clé d'idempotence sur les écritures (même schéma que `v008` et `v009`).

**Variables** : module `templates/variables`. Un enum `TemplateVariable` couvre les 8 variables, routage par `match` exhaustif. Le rendu est une fonction pure (texte + contexte → texte rendu + variables non résolues). Le contexte porte date, heure et fuseau, ce qui rend les tests déterministes.

**Frontière** : aucune variable n'est propre à un provider ; le cœur ne nomme aucun provider.

## Contrat d'API

Fichiers `api/contract/template_requests.rs` et `template_responses.rs`, un handler par opération.

Requêtes (`TemplateRequest`, `#[serde(tag = "op")]`) :
`ListTemplates`, `GetTemplate`, `CreateTemplate { name, subject, body, idempotency_key }`, `UpdateTemplate`, `DeleteTemplate`, `RenderTemplate { template, context }`.

`RenderContext` : `contact` (nom, prénom, email, optionnel), `account_name`, `account_email`, `subject`, `now`. Fourni par Flutter ; le moteur ne devine rien.

Réponses : `Templates`, `Template`, `Rendered { subject, body, unresolved }`.

Erreurs typées : `TemplateNameEmpty`, `TemplateNameTaken`, `TemplateNotFound`, traduites dans `i18n/engine_errors.dart`.

Événement `TemplatesChanged`, émis après chaque écriture seulement.

Le pont FFI transporte l'enveloppe existante ; peu ou pas de nouvelle fonction exposée. Décision à consigner dans `docs/DECISIONS-AGENT.md`.

## Interface Flutter

- `engine/template_admin.dart` : `EngineTemplate` et `TemplateAdmin` (liste, crée, modifie, supprime, rend).
- `state/templates.dart` : liste chargée au démarrage, rafraîchie par `TemplatesChanged` (route dans `event_routes.dart`). Pas de copie persistée côté app.
- `ui/templates_settings.dart` (ajouté à `settings_panel.dart`) : liste, éditeur nom/objet/corps, barre de variables cliquables insérées au curseur, suppression avec confirmation, erreurs traduites.
- `ui/template_picker.dart` : sélecteur avec recherche, ouvert par le bouton « Modèle » de la barre de formatage ou par `/`. Le `/` ne se déclenche qu'en début de ligne ou après un espace (pas dans `http://`). Échap ferme sans insérer.
- À l'insertion : l'app construit le `RenderContext` (destinataire principal via `state/contacts.dart`, identité d'envoi, objet courant, date et heure), appelle `RenderTemplate`, insère le corps au curseur et renseigne l'objet s'il est vide.
- Variables non résolues : laissées en clair, surlignées, avec un message du type « 2 variables à compléter ».
- `ui/composer.dart` est déjà volumineux : le nouveau code est extrait dans des fichiers dédiés.
- Si le destinataire n'est pas encore saisi, `contact.*` reste visible ; pas de rendu différé en V1.
- Textes via `i18n`.

## Tests

Moteur (boîte noire) : chaque variable ; espaces tolérés ; inconnue ou sans valeur laissée visible et listée dans `unresolved` ; `{{` non fermé sans panique ; CRUD ; nom unique et nom vide refusés ; migration sur base existante ; idempotence de `CreateTemplate` ; test de contrat (forme sérialisée figée) ; `TemplatesChanged` émis après écriture et jamais après lecture ; aucun contenu de modèle dans journaux ni erreurs.

Flutter : tests unitaires de `TemplateAdmin` et de l'état ; tests de widget du sélecteur (recherche, Échap) et du déclenchement de `/`.

## Découpage

1. Vocabulaire, migration, dépôt, rendu des variables, tests moteur.
2. API (requêtes, réponses, handlers, événement) et test de contrat.
3. Couche Flutter : `TemplateAdmin`, état, écran de gestion.
4. Intégration à l'éditeur : bouton, raccourci `/`, surbrillance des variables non résolues.

Les étapes 1 et 2 sont séquentielles ; 3 et 4 peuvent avancer en parallèle de 2 une fois le contrat figé.

## Règles de travail

Normes du projet (fonction ≤ 30 lignes, aucun commentaire dans un corps de fonction, pas de `unwrap` hors tests). `./ci.sh` uniquement avec l'accord de l'utilisateur. Commits au nom de l'utilisateur, sans trailer d'attribution.

## Hors périmètre (V1)

Synchro entre appareils, modèles par compte, rendu différé des variables `contact.*`, variables personnalisées, modèles avec pièces jointes.
