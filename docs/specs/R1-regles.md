# R1 — Règles « condition → action »

Spécification du système de règles automatiques du moteur et de son raccordement à l'app Flutter.
Référence : §11.3 (« Règles et filtres automatiques »), §12 (tout ce que l'app fait existe d'abord dans le moteur, testable sans écran), D-06 (le cœur ne nomme aucun provider), invariant 18 (idempotence), invariant 28 (réponse immédiate, serveur ensuite).

## 1. Objectif et décisions validées

Une règle s'exécute automatiquement sur les nouveaux messages et déclenche des actions. L'utilisateur la crée et la modifie depuis l'app ; toute la logique est dans le moteur.

| Décision | Choix |
|---|---|
| Quand | À l'arrivée des nouveaux messages (synchro incrémentale). Pas de rattrapage manuel en v1. |
| Combinaison | « OU de groupes ET » : une règle a une ou plusieurs alternatives (OU) ; chaque alternative est une suite de conditions reliées par ET ; chaque condition est inversable. Pas de parenthèses en v1. |
| Portée | Par règle : un ou plusieurs comptes, ou tous les comptes. |
| Architecture | Évaluateur pur dans le moteur, branché sur la synchro, actions soumises par la file existante. |
| Boîtes concernées | Toutes sauf Envoyés, Brouillons, Corbeille et Spam. |

Hors périmètre v1 : condition sur le corps complet, conditions sur en-têtes bruts, groupes imbriqués, supprimer définitivement, transférer, répondre automatiquement, appliquer aux messages existants.

## 2. Types (`mira-vocabulary`)

JSON étiqueté (`type`, snake_case), comme `ActionKind`.

- `Rule` : `id`, `name`, `enabled`, `scope` (`AllAccounts` | `Accounts(Vec<AccountId>)`), `alternatives: Vec<ConditionGroup>`, `actions: Vec<RuleAction>`, `schema_version`. L'ordre des règles n'est pas dans `Rule` (porté par le stockage).
- `ConditionGroup` : `conditions: Vec<Condition>` (ET).
- `Condition` : `negated: bool`, `test: ConditionTest`.
- `TextMatcher` : opérateur (`contains`, `equals`, `starts_with`, `ends_with`, `regex`, `in_list`) et valeur ou liste de valeurs ; casse ignorée.

### 2.1 Catalogue de conditions (`ConditionTest`)

| Famille | Variantes |
|---|---|
| Expéditeur | `Sender { field: address\|name\|domain, matcher }` |
| Destinataires | `Recipient { which: to\|cc\|bcc, field, matcher }`, `MeInTo`, `MeInCcOnly`, `RecipientCount { cmp, n }` |
| Répondre à | `ReplyTo { field, matcher }` |
| Contenu | `Subject { matcher }`, `Preview { matcher }` |
| Dates | `ReceivedBefore`, `ReceivedAfter`, `OlderThanDays` |
| Taille | `SizeCompare { cmp, bytes }` |
| Pièces jointes | `HasAttachment`, `AttachmentName { matcher }`, `AttachmentType { matcher }` |
| Drapeaux | `IsRead`, `IsFlagged`, `IsAnswered`, `IsForwarded` |
| Nature | `IsNewsletter` (en-tête de désabonnement), `IsReply` (`In-Reply-To`), `InExistingThread` |
| Emplacement | `InMailbox { mailbox }`, `FromAccount { account }` |

### 2.2 Catalogue d'actions (`RuleAction`)

| Famille | Variantes | Traduction |
|---|---|---|
| Drapeaux | `MarkRead { read }`, `MarkFlagged { flagged }` | `ActionKind::MarkRead`, `MarkFlagged` |
| Classement | `MoveTo { mailbox }`, `AddToMailbox`, `RemoveFromMailbox`, `Archive` | `Move`, `AddToMailbox`, `RemoveFromMailbox`, `Archive` |
| Nettoyage | `Trash`, `MarkSpam { spam }` | `Trash`, `MarkSpam` |
| Notification | `SuppressNotification` | propre aux règles |
| Enchaînement | `StopProcessing` | propre aux règles |

## 3. Stockage (SQLite, par profil)

- `rules` : `id`, `name`, `enabled`, `position`, `schema_version`, `definition` (JSON : alternatives et actions), `created_at`, `updated_at`.
- `rule_accounts` : `rule_id`, `account_id`, une ligne par compte ; aucune ligne si la règle vise tous les comptes (colonne `scope_all` dans `rules`). Supprimer un compte retire ses rattachements ; une règle dont la portée devient vide est invalide, jamais « tous les comptes ».
- `rule_pending` : `message_id`, écrite dans la transaction d'ingestion, supprimée une fois les actions soumises (reprise sur crash).
- Les règles sont de la configuration : migrées sans perte. `rule_pending` est transitoire.

## 4. Évaluation (`mira-core/src/rules`)

Fonction pure, sans I/O.

- Entrée : règles ordonnées et `MessageContext` = `MessageSummary` + adresses du compte (pour « moi ») + pièces jointes (nom, type) + indicateur « fil déjà existant » + heure courante.
- Sortie : `RuleOutcome { actions: Vec<ActionKind>, silenced: bool, matched: Vec<RuleId> }`.
- Parcours : règles activées, valides, par `position`, dont la portée contient le compte du message. Une règle se déclenche si au moins une alternative a toutes ses conditions vraies (inversions appliquées). Ses actions s'ajoutent dans l'ordre. `StopProcessing` interrompt le parcours. En cas de conflit, la dernière règle l'emporte sur un même drapeau.
- Expressions régulières : crate `regex` (temps linéaire), compilées et plafonnées en taille à l'enregistrement.

## 5. Validation (à l'enregistrement, erreur explicite)

- Au moins une alternative non vide et au moins une action.
- Expression régulière invalide ; plafonds dépassés (alternatives, conditions, actions, taille des valeurs).
- Action sur une boîte : portée = un seul compte, boîte appartenant à ce compte.
- Actions contradictoires dans une même règle (par exemple `Trash` et `Archive`).
- Comptes de la portée inexistants.
- Si une boîte visée disparaît ensuite, la règle est **invalide** (statut calculé à la lecture) : non exécutée, signalée dans l'app avec la raison.

## 6. Intégration à la synchro et à la file d'actions

- Accroche : `sync/strategy/headers.rs`, `announce_batch`, pour chaque `Ingested::Created` en mode `Announce::Live`. Jamais en `Announce::Silent` (téléchargement initial, resynchronisation).
- L'évaluation précède l'émission de `NewMessageReceived`, qui reçoit un champ `silenced` (faux par défaut).
- Les actions passent par `actions::submit`, avec la clé d'idempotence `(règle, message, rang de l'action)`. Elles héritent du délai d'annulation (`UNDO_DELAY_MILLIS`) et de l'effet local immédiat.
- Reprise sur crash : `rule_pending` est rejouée au redémarrage ; les clés d'idempotence rendent le rejeu sûr.
- Boîtes concernées : toutes sauf Envoyés, Brouillons, Corbeille, Spam (par rôle).

## 7. API (domaine `RuleRequest`)

`ListRules`, `GetRule`, `CreateRule`, `UpdateRule`, `DeleteRule`, `SetRuleEnabled`, `ReorderRules`, `ValidateRule` (sans enregistrer), `PreviewRule` (nombre de messages existants touchés et exemples, sans rien modifier), `ListRuleCatalog` (catalogue de conditions et d'actions : le moteur reste la source de vérité).
Les écritures portent une clé d'idempotence. Toute règle lue porte son statut (valide, ou invalide avec la raison).
Événements : `RuleChanged`, `RuleApplied { rule, message }`. Aucun secret dans les événements ni les erreurs.

## 8. Pont et app

- Pont : aucun changement, la requête JSON `{domaine, opération, données}` est générique ; la section 6 ajoute le domaine `rule` au routeur. Test d'intégration dans `bridge/tests/engine_bridge.rs`.
- App : `engine/rule_admin.dart` (sur le modèle de `label_admin.dart`), modèles Dart de `Rule` et associés, `ui/rules_settings.dart` (liste réordonnable, activation, badge « invalide »), éditeur (blocs ET séparés par OU, inversion par condition, actions, aperçu), libellés dans `i18n/`, prise en compte de `RuleChanged` et de `silenced`.

## 9. Tests

- Évaluateur : chaque condition et chaque action, ET, OU, inversion, portée par compte, `StopProcessing`, conflits.
- Validation : chaque cas de refus de la section 5, règle devenue invalide.
- Intégration (testkit) : un nouveau message déclenche l'action ; le téléchargement initial ne déclenche rien ; crash avant soumission repris sans doublon ; rejeu idempotent ; absence de secrets.
- Contrat : test de contrat du domaine `rule`.
- Pont : création, liste et suppression de bout en bout.
- App : tests de modèles, d'éditeur et de liste.

## 10. Livraison en quatre lots

1. Moteur : types, stockage, évaluateur, validation.
2. Moteur : intégration synchro, API, événements, reprise sur crash.
3. Pont : test d'intégration.
4. App : modèles, `rule_admin`, liste et éditeur.

Les lots 1 à 3 sont testables sans écran.
