# Historique des conversations avec l'assistant IA — design

Date : 2026-11-01 · Statut : implémenté · Chemin : architectural (nouveau stockage, contrat 1.3 → 1.4, nouvelle interface)

## 1. Objectif

Retrouver dans l'app, comme dans ChatGPT, **plusieurs conversations passées** avec l'assistant : liste, reprise, renommage, suppression. Tout reste **local** (pas de synchronisation ni de compte) et respecte la logique de confidentialité de l'IA déjà en place.

### État actuel

- Le fil est une `List<ChatMsg>` en mémoire dans `ChatController` : fermer l'app ou le panneau efface tout.
- Le journal IA (`ai_journal`, migrations v018/v019) ne contient **volontairement aucun contenu** ; il ne peut pas servir d'historique et ne change pas.
- Le moteur possède une base SQLite par profil (`secure_delete=ON`, migrations numérotées, contrat versionné à 1.3). L'app n'a que `shared_preferences`.

### Décisions prises avec l'utilisateur

| Sujet | Choix |
|---|---|
| Forme | Plusieurs conversations listées et reprenables |
| Contenu enregistré | Ce qui est affiché : questions, réponses (repères restitués), pastilles d'outils, identifiants des sources. **Pas** les parties de contexte ni les `tool_result` envoyés au modèle |
| Réglage | Interrupteur global « Conserver l'historique », **activé par défaut**, avec « Effacer tout » |
| Stockage | Moteur, un **document JSON par conversation** (métadonnées en colonnes, payload opaque) |

### Hors périmètre (YAGNI)

Titre généré par IA, recherche dans le contenu, synchronisation entre appareils, export, dossiers, épinglage, historique en démo (sans moteur).

## 2. Moteur : stockage et contrat

### Stockage — migration `v020_ai_conversations.sql`

Table `ai_conversations` :

| Colonne | Rôle |
|---|---|
| `id` TEXT PK | identifiant fourni par l'app (UUID) |
| `title` TEXT | 80 caractères au plus |
| `created_at`, `updated_at` | dates ; index sur `updated_at DESC` |
| `payload_version` INTEGER | version de la forme du payload |
| `payload` TEXT | texte JSON (objet), opaque pour le moteur |

L'interrupteur (global, `true` par défaut) est rangé dans `engine_settings` (v011) sous la clé `ai_history_enabled` (`"1"` / `"0"`), lue et écrite par des méthodes dédiées du dépôt : la structure `EngineSettings` et le contrat des réglages ne changent pas. Dans le contrat, `payload` est une **chaîne** (texte JSON) : le vocabulaire n'a pas `serde_json`, et le moteur reste ainsi réellement opaque ; il vérifie seulement que c'est un objet JSON.

### Contrat 1.4

Un handler par opération dans `api/handlers/ai/conversations/`, routage par `match` exhaustif.

| Requête | Effet | Réponse | Risque (`risk_of`) |
|---|---|---|---|
| `ListAiConversations { limit, before }` | métadonnées seules, plus récentes d'abord | `Conversations` | Sensitive |
| `GetAiConversation { id }` | conversation complète | `Conversation` | Sensitive |
| `SaveAiConversation { id, title, payload_version, payload }` | upsert idempotent | `ConversationMeta` | Sensitive |
| `RenameAiConversation { id, title }` | renomme (l'ordre ne change pas) | `ConversationMeta` | Sensitive |
| `DeleteAiConversation { id }` | supprime | `Done` | Sensitive |
| `ClearAiConversations` | supprime tout | `Done` | Sensitive |
| `SetAiHistoryEnabled { enabled }` | interrupteur | `HistoryEnabled` | Sensitive |
| `GetAiHistoryEnabled` | lit l'interrupteur | `HistoryEnabled` | Read |

`ToolRisk` n'a pas de niveau « Write » (`Read`, `Reversible`, `Confirmed`, `Sensitive`) : toutes les écritures du domaine IA sont `Sensitive`, comme `ClearAiJournal`. Les lectures de conversations sont aussi `Sensitive`, pour que l'historique ne puisse jamais être exposé au modèle par un futur outil ; seul `GetAiHistoryEnabled` est `Read`. Le catalogue d'outils reste à cinq outils. Le contrat passe de 1.3 à 1.4.

### Bornes et erreurs

- Titre ≤ 80 caractères, sinon `invalid_input.ai_conversation_title`.
- Payload ≤ 2 Mio, sinon `invalid_input.ai_conversation_too_large` ; doit être un objet JSON valide, sinon `invalid_input.ai_conversation_payload`.
- Rétention : 200 conversations ; l'enregistrement d'une 201e supprime la plus ancienne par `updated_at`. Pas de limite d'âge.
- `Save` avec l'historique désactivé : `incompatible_state.ai_history_disabled` (l'app vérifie aussi, le moteur reste la garde).
- `Get`/`Rename`/`Delete` d'un `id` inconnu : `not_found.ai_conversation`.
- `List` : `limit` de 1 à 200, sinon `invalid_input.parameter` ; `before` est l'`id` de la dernière ligne reçue (ordre `updated_at DESC, rowid DESC`).
- Le titre est compté en caractères (pas en octets), vide ou blanc refusé.

### Confidentialité

- Le payload n'apparaît dans **aucun** événement, `tracing`, erreur ni `Debug`. `ai_journal` reste sans contenu.
- Désactiver l'historique ne supprime rien ; les conversations existantes restent visibles et supprimables.
- Suppression sûre grâce à `secure_delete=ON`.

## 3. Pont et état Flutter

Le pont est générique (`bridge/src/api/engine.rs::execute(request_json)` ; côté Dart `AiAdmin._send` → `req('ai', op, data)`) : les opérations 1.4 passent sans nouveau code de pont ni régénération `frb` (vérifié).

1. **`AiHistoryAdmin`** (`engine/ai_history_admin.dart`), classe séparée d'`AiAdmin` : `list`, `get`, `save`, `rename`, `delete`, `clearAll`, `setEnabled`.
2. **Modèles** (`engine/ai_conversation_models.dart`) : `AiConversationMeta { id, title, updatedAt }` ; payload v1 = objet `{"messages": [...]}`, le message d'accueil exclu (il est recréé à l'ouverture), chaque message portant (`role`, `text`, `draft`, `refs`, `sources`, `tools[{id, name, summary, state, results, deniedKind}]`, `failed`, `note`, `maskedCount`, `fromEngine`). `shown` n'est pas enregistré ; un message non terminé n'est jamais enregistré. Version inconnue ou ligne corrompue → conversation « illisible », supprimable.
3. **`ChatHistoryState`** (`state/chat_history_state.dart`), `ChangeNotifier` calqué sur `AiJournalState` : pages de 50, `load`, `more`, `rename`, `delete`, `clearAll`, génération pour écarter les lectures périmées, erreurs via `engineErrorText`.
4. **`ChatController`** : `conversationId` courant (UUID créé à la première question), `startNew()`, `open(id)`, sauvegarde automatique dans `_finish` (une par tour terminé, sans minuterie). Pas de sauvegarde en démo, avec l'historique coupé, ou si le fil ne contient que l'accueil. Titre = première question nettoyée, tronquée à 80, fixe sauf renommage.

### Reprise et cas limites

- Le modèle reçoit le fil reconstruit depuis le texte visible ; les `tool_result` passés ne sont pas renvoyés, comme dans une même session aujourd'hui.
- Sources dont le mail n'existe plus : affichées comme inconnues (comportement existant).
- Changer de conversation pendant une génération : désactivé dans l'interface.
- Échec de `save` : jamais bloquant, avertissement discret, nouvelle tentative au tour suivant.
- Historique désactivé en cours de conversation : l'existant reste, les nouveaux tours ne sont plus enregistrés.
- Suppression de la conversation ouverte : remplacée par un fil neuf.

## 4. Interface

L'assistant est un volet unique (plein écran sous 1100 px) sans colonne latérale. L'historique est donc une **vue qui remplace le fil dans le même volet**, comme le « Journal » des réglages (`showJournal`).

- **En-tête** : bouton icône « Historique » (masqué en démo) à côté de « Nouvelle conversation » (existant, appelle `clearChat`). Il bascule `showHistory` (état local).
- **`ui/chat_history_view.dart`** : remplace bandeau de contexte, fil et saisie. Lignes de 48 px minimum : titre (ellipse), date relative (`data/format.dart`), conversation ouverte surlignée, menu de ligne toujours visible (Renommer, Supprimer). Clic = ouvrir et revenir au fil ; lignes grisées pendant une génération (info-bulle). Pied : « Afficher plus » (pages de 50), « Tout effacer » avec confirmation.
- **Renommer** : boîte de dialogue, champ limité à 80. **Supprimer** : confirmation sur le modèle de `_ClearDialog` du journal.
- **États vides** : « Aucune conversation enregistrée » ou « L'historique est désactivé » avec lien vers les réglages.
- **Bandeau dans le fil** quand l'historique est coupé : « Historique désactivé : cette conversation ne sera pas enregistrée ».
- **Réglages IA** (`ai_settings.dart`) : section « Historique des conversations » — interrupteur (activé par défaut), texte « Enregistre sur cet appareil le texte affiché des conversations. Les contenus envoyés au modèle en coulisses ne sont pas conservés. », bouton « Effacer tout l'historique » avec confirmation.
- Chaînes via `t()` + entrées `i18n/lang/en.dart` ; `ValueKey` selon la convention (`chat-history-open`, `chat-history-row-<id>`…) ; composants existants uniquement (`Btn`, `IBtn`, `Mono`, `RuleChip`, `DashedBox`). Icône « historique » à vérifier dans `theme/icons.dart`, sinon à ajouter au même jeu.

## 5. Tests

**Moteur** (fichiers séparés, normes `tools/check-norms`) : idempotence du `Save` (une ligne, `updated_at` avance) ; liste (ordre, pagination `before`, pas de payload) ; `Get` inconnu ; renommage, suppression, effacement ; bornes (titre, 2 Mio, JSON invalide, éviction à 201) ; interrupteur (défaut activé, `Save` refusé coupé, la coupure ne supprime rien) ; isolation par profil ; absence du payload dans erreur, `Debug`, événements et `ai_journal` ; instantanés de contrat 1.4, version à l'ouverture (invariant 33), `risk_of`.

**App** : aller-retour de sérialisation (états d'outils compris) et version inconnue ; `ChatHistoryState` (pagination, lecture périmée, effacement, erreurs) ; `ChatController` (une sauvegarde par tour terminé, aucune pendant la génération, ni en démo, ni coupé, ni accueil seul ; titre ; `open` ; suppression de la courante ; échec non bloquant puis nouvelle tentative) ; widgets (bouton masqué en démo, lignes grisées en génération, dialogues, états vides, interrupteur, largeur étroite).

## 6. Découpage

1. Moteur : migration v020 + repository.
2. Moteur : contrat 1.4, handlers, réglage, `risk_of`, instantanés.
3. App : `AiHistoryAdmin` + modèles (faux client, parallélisable avec 2).
4. App : `ChatHistoryState` + `ChatController`.
5. App : interface, réglages, i18n.
6. Documentation : nouvelle sous-section de `ARCHITECTURE-IMPLEMENTEE.md` §6 (contrat 1.4).

Dépendances : 2 ← 1 ; 5 ← 4. Un commit par lot, tests d'abord. Tests ciblés par lot (`cargo test -p …`, `flutter test <fichiers>`) ; `./ci.sh` seulement avec l'accord de l'utilisateur.

## 7. Points levés à l'écriture du plan

- `engine_settings` : clé dédiée `ai_history_enabled`, sans toucher `EngineSettings`.
- Pont : aucun changement (vérifié).
- Icône « historique » : `clock` existe déjà dans `theme/icons.dart`.
- « Tout effacer » depuis l'historique remet aussi le fil ouvert à neuf (sinon le prochain tour ressusciterait la conversation effacée).
