# Mémoire de l'IA (souvenirs vectoriels) — design

Date : 2026-11-15 · Statut : à approuver · Branche : `feat/ia-memoire` (worktree `../MIRA-memoire`) · Chemin : architectural

## 1. Intention

L'IA du chat n'a aucune mémoire entre deux conversations. Objectif : lui donner une **mémoire durable, locale et contrôlée par l'utilisateur**, pour qu'elle se souvienne de faits, de préférences et d'actions passées, qu'on puisse lui « apprendre » des choses, et (lot 2) qu'elle rédige dans le style de l'utilisateur.

## 2. Buts, non-buts, anti-buts

- **Buts** : souvenirs explicites (retenir / retrouver / oublier) ; recherche par sens (vecteurs) ; mémoire désactivée par défaut, activée par l'utilisateur ; choix du moteur d'embeddings.
- **Non-buts (lot 1)** : apprentissage automatique du style depuis les mails envoyés (lot 2) ; indexation de toute la boîte ; synchronisation entre appareils.
- **Anti-buts** : contenu de mails envoyé à un tiers sans consentement explicite ; souvenir créé en silence ; mémoire qu'on ne peut ni lister ni effacer ; secret dans un souvenir.

## 3. Contraintes

Règles du `CLAUDE.md` : zéro `unsafe`/`unwrap` hors tests, fonction ≤ 30 lignes, fichier ≤ 6 fonctions, frontière cœur/providers, pas de plugin dynamique, tests contre `127.0.0.1` seulement. Le noyau ne nomme aucun fournisseur d'IA.

## 4. Approches

| | Approche | Compromis |
|---|---|---|
| **A (retenue)** | Embeddings locaux via le serveur llama.cpp existant (`/v1/embeddings`), modèle GGUF épinglé téléchargé à la demande | Réutilise téléchargement vérifié et gestion de processus ; 100 % local ; second modèle en mémoire |
| B | Embeddings dans le processus (`fastembed`/`ort`/`candle`) | Pas de serveur, mais grosse dépendance, second système de téléchargement |
| C (option) | Embeddings par le fournisseur d'IA configuré | Léger, mais le contenu part chez un tiers ; avertissement explicite requis |

Choix de l'utilisateur : **A par défaut, C en option explicite** ; B écartée. Stockage : table SQLite + recherche par similarité en Rust (une base vectorielle dédiée n'est pas justifiée à cette échelle ; `sqlite-vec` possible plus tard derrière la même interface).

## 5. Décisions

| # | Décision | Raison |
|---|---|---|
| M1 | Mémoire **désactivée par défaut** ; réglage `memory.enabled` + `memory.embedder` (`local` / `provider`). | Consentement ; pas de téléchargement imprévu. |
| M2 | Activer le mode `local` propose le téléchargement du modèle d'embeddings (progression, annulation, reprise, SHA-256) via `ai/local/download*`. | Brique existante. |
| M3 | Souvenir = `{ id, text, kind (fact/preference/action), source (user/ai), created_at, embedding, model_id }`, par profil. | Traçabilité, ré-indexation possible si le modèle change. |
| M4 | Outils du chat : `remember`, `recall`, `forget`. `remember` d'initiative de l'IA passe par le **consentement** existant ; l'utilisateur voit la carte du souvenir. | Pas de souvenir silencieux. |
| M5 | `recall` est aussi appelé par le moteur en début de tour (top-k borné) ; les souvenirs rappelés vont dans le contexte, signalés comme tels. | Mémoire utile sans que le modèle y pense. |
| M6 | Changer de modèle d'embeddings marque les vecteurs obsolètes ; ré-indexation en tâche de fond. | Les vecteurs de modèles différents sont incomparables. |
| M7 | Écran « Mémoire » : liste, recherche, modification, suppression unitaire et totale ; effacement du profil (`storage/erasure`) inclut la mémoire. | Contrôle total. |
| M8 | Filtre de secrets avant stockage (réutiliser le masquage existant `ai/masking`) ; refus si détection. | Aucun secret en base de souvenirs. |
| M9 | Contrat : nouveaux types dans `mira-vocabulary`, opérations dans la façade, relais par le pont ; golden tests du protocole mis à jour. | Même chemin que `DraftProposed`. |

## 6. Découpage en lots

1. **Lot 1a** : stockage (migration, dépôt, similarité cosinus) + `Embedder` (interface) + implémentation locale + téléchargement du modèle.
2. **Lot 1b** : outils `remember`/`recall`/`forget`, rappel en début de tour, consentement, contrat.
3. **Lot 1c** : écran Flutter « Mémoire » et réglages.
4. **Lot 2** (spec séparée) : style de rédaction à partir des mails envoyés.

## 7. Laissé au réalisateur

Choix exact du modèle d'embeddings (multilingue, ≤ 300 Mo, GGUF) à épingler avec taille et SHA-256 ; valeur de k et seuil de similarité ; dimension des vecteurs ; format de sérialisation (BLOB `f32`) ; option de serveur dédié ou partagé avec le modèle de chat.
