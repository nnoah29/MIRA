# Mémoire de l'IA, lots 1b et 1c (moteur, chat, interface) — design

Date : 2026-11-15 · Statut : approuvé par l'utilisateur (« attaque les lots suivants et termine tous les lots 1 ») · Suite de `2026-11-15-ia-memoire-design.md` (décisions M1–M9, lot 1a livré) · Branche `feat/ia-memoire`

## 1. Intention

Rendre la mémoire utilisable de bout en bout : l'utilisateur l'active, choisit où les embeddings sont calculés, voit et corrige ses souvenirs ; l'assistant s'en sert dans le chat et propose d'en ajouter, sans jamais écrire en silence.

## 2. Décisions

| # | Décision | Raison |
|---|---|---|
| B1 | Réglages persistés dans `engine_settings` : `ai.memory.enabled` (faux par défaut), `ai.memory.embedder` = `local` ou `provider`, et pour `provider` la connexion et le nom du modèle d'embeddings. | M1. |
| B2 | Mode `local` : la fiche du runtime local (`AiLocalRuntime`) porte un champ facultatif `embedding { model, server_args }` rempli par `mira-ai-local`. Le cœur ne nomme rien. | Le cœur reçoit le modèle et les arguments par la fiche, comme le serveur de chat. |
| B3 | Mode `provider` : `HttpEmbedder` sur l'adresse et la clé de la connexion choisie ; l'identifiant de modèle est qualifié `<fournisseur>:<modèle>`. Un avertissement de confidentialité est montré par l'app avant activation. | Contrat noté à la revue du lot 1a (Important 3). |
| B4 | Installation du modèle local : opération en arrière-plan avec progression et annulation, comme `InstallLocalModel`, qui installe le serveur s'il manque puis le modèle d'embeddings sous `<profil>/ai-memory/`. Un seul emplacement d'installation partagé avec le chat. | Brique existante ; une seule installation à la fois. |
| B5 | Le `MemoryStore` est construit à la demande selon les réglages et gardé par le moteur ; un changement de réglage le jette ; `shutdown_within` arrête le serveur d'embeddings. Un changement d'embedder lance une réindexation à la demande (`ReindexMemories`). | M6. |
| B6 | Nouveau type de contenu `ContentKind::Memory` (`memory`) : les souvenirs envoyés au modèle (rappel automatique et résultats de `recall_memory`) sont un contenu comme un autre, soumis au consentement par connexion. Aucun consentement pour une connexion locale. | Les souvenirs sont des données personnelles envoyées à un tiers : le mécanisme de types autorisés existe pour ça. |
| B7 | Outils du chat, annoncés en mode Assistant seulement et seulement si la mémoire est activée : `recall_memory {query}` (lecture, type `memory`), `remember {text, kind}` (proposition), `forget_memory {id}` (proposition qui lit le texte pour la carte). Une proposition n'écrit rien : le moteur émet `ChatDelta::MemoryProposed` et l'app applique par `RememberMemory` / `ForgetMemory` quand l'utilisateur clique. | M4 : pas de souvenir silencieux, même schéma que `compose_draft` et `propose_action`. |
| B8 | Rappel automatique (M5) : au début d'un tour d'Assistant, le moteur cherche au plus 5 souvenirs proches du dernier message de l'utilisateur et les joint comme partie de contexte `memory`, rendue par le moteur et masquée comme le reste. Un échec de la recherche n'empêche jamais la discussion. | M5. |
| B9 | API : `GetMemoryStatus` (lecture), `SetMemorySettings`, `InstallMemoryModel`, `ListMemories`, `SearchMemories`, `RememberMemory`, `UpdateMemory`, `ForgetMemory`, `ForgetAllMemories`, `ReindexMemories`. Contrat 1.10 → 1.11. | M9. |
| B10 | Un refus du modèle d'embeddings pour « texte trop grand » n'est pas réessayable. | Revue 1a, Important 6. |

## 3. Interface (lot 1c)

- **Réglages → Assistant → Mémoire** : interrupteur ; choix « Sur cet appareil » (téléchargement du modèle avec progression et annulation) ou « Via mon fournisseur » (connexion et modèle, avertissement de confidentialité) ; compteur de souvenirs ; bouton « Réindexer » si des souvenirs sont périmés ; « Tout oublier » avec confirmation.
- **Liste des souvenirs** : recherche, genre, origine, modification du texte, suppression.
- **Chat** : carte « Retenir ceci ? » (texte, genre) avec « Retenir » et « Ignorer » ; carte « Oublier ceci ? » avec le texte du souvenir ; pastille d'outil « Mémoire » dans les puces d'outils.
- Textes en français et anglais dans `app/lib/i18n`.

## 4. Non-buts

Style de rédaction (lot 2) ; indexation de la boîte ; synchronisation entre appareils ; déduplication des souvenirs (noté en différé à la revue du lot 1a).

## 5. Laissé au réalisateur

Libellés exacts, forme visuelle des cartes (suivre `chat_draft_card.dart` et `chat_mail_cards.dart`), taille de page des listes.
