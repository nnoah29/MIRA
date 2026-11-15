# Mémoire de l'IA, lots 1b et 1c — plan d'implémentation

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans (choix de l'utilisateur : exécution native) task-by-task. Steps use checkbox (`- [ ]`) syntax.

**Goal :** brancher la mémoire du lot 1a au moteur, à l'API, au chat et à l'interface Flutter.

**Architecture :** réglages dans `engine_settings` ; `MemoryRuntime` dans `AiState` construit le `MemoryStore` selon les réglages (embedder local ou fournisseur) ; opérations de contrat 1.11 ; outils du chat en propositions (rien n'est écrit sans clic) ; rappel automatique en partie de contexte `memory` ; app Flutter : réglages, liste, cartes de chat.

**Tech Stack :** Rust 1.99, `mira-core`/`mira-vocabulary`/`mira-ai-local`, Flutter/Dart (`app/`).

**Spec :** `docs/superpowers/specs/2026-11-15-ia-memoire-lots-1b-1c-design.md` (B1–B10) et `2026-11-15-ia-memoire-design.md`.

## Global Constraints

- Mêmes règles que le plan du lot 1a (normes, architecture, aucun `unwrap` hors tests, tests sur `127.0.0.1`, tests ciblés, un commit par tâche sans trailer, `./ci.sh` seulement sur accord — accordé le 2026-11-15 pour la vérification complète en fin de lots).
- Modèle des changements : le commit `9cbc836` (outil `show_mails`, côté moteur) et `5be42dc` (cartes dans l'app) sont le gabarit des couches traversées.
- Contrat : `CONTRACT_VERSION` 1.10 → 1.11, `mira-protocol` et ses golden tests mis à jour, rien de spécifique au pont (il relaie du JSON).
- Dart : `flutter analyze` propre, tests `flutter test` ciblés sur les fichiers touchés, i18n fr/en complète (`tools/i18n-audit`).

## Review Focus

- Une mémoire **désactivée** n'annonce aucun outil, n'ajoute aucun contexte et ne lance aucun serveur (T6, T7).
- Un **échec d'embedder** (serveur local absent, clé refusée) ne casse jamais une discussion (T7).
- Une proposition `remember` **n'écrit rien** tant que l'app n'a pas appelé `RememberMemory` (T6).
- Un **secret** dans `remember` est refusé au moment de la proposition et à l'écriture (T5, T6).
- Changer d'**embedder** ne perd aucun souvenir : ils redeviennent cherchables après `ReindexMemories` (T4, T5).

---

## Lot 1b — moteur

### Task 1: Vocabulaire et contrat 1.11
**Files:** `mira-vocabulary/src/ai/memory_settings.rs` (nouveau), `.../ai/memory.rs` (ajouts), `.../ai/content.rs` (`ContentKind::Memory`), `.../ai/delta.rs` (`MemoryProposed`), `.../ai/local_runtime.rs` (`embedding`), `.../contract/version.rs`, `mira-ai-local/src/pinned.rs`, fixtures `tests/local_support/mod.rs` et `tests/local_ops_support/build.rs` ; tests `mira-vocabulary/tests/ai_memory_settings.rs`.
**Interfaces — Produces :** `MemoryEmbedder { Local, Provider { connection: AiConnectionId, model: String } }` (serde `kind`), `MemorySettings { enabled: bool, embedder: MemoryEmbedder }` (défaut : désactivée, `Local`), `MemoryStatus { settings, model_installed: bool, count: u64, stale: u64 }`, `MemoryProposalAction { Remember, Forget }`, `MemoryProposal { action, text, kind: MemoryKind, id: Option<String> }`, `ChatDelta::MemoryProposed { id: String, proposal: MemoryProposal }`, `ContentKind::Memory` (`as_str` = `memory`), `AiLocalEmbedding { model: AiLocalModel, server_args: Vec<String> }`, `AiLocalRuntime.embedding: Option<AiLocalEmbedding>` (`#[serde(default)]`).
- [ ] Tests : aller-retour JSON de chaque type ; `MemorySettings::default()` désactivée en `Local` ; `ContentKind::Memory.as_str() == "memory"` ; `CONTRACT_VERSION == 1.11`.
- [ ] Rouge, implémenter, vert ; corriger les fixtures (`embedding: None`) ; `mira-ai-local` remplit `embedding` avec `embedding_model()` et `embedding_server_args()` (test `descriptor`).
- [ ] Commit `feat(ia): types de réglages et de propositions de la mémoire, contrat 1.11`.

### Task 2: Réglages persistés
**Files:** `mira-core/src/storage/repositories/engine_settings/memory.rs`, test `tests/storage_ai_memory_settings.rs`.
**Interfaces — Consumes :** `MemorySettings` (T1). **Produces :** `Tx::ai_memory_settings() -> Result<MemorySettings, MiraError>` (défaut si absent ou illisible), `Tx::set_ai_memory_settings(&MemorySettings) -> Result<(), MiraError>`.
- [ ] Tests : défaut désactivé ; aller-retour `Local` et `Provider` ; valeur corrompue → défaut.
- [ ] Rouge, implémenter (JSON dans une clé `ai.memory`), vert, commit `feat(stockage): réglages de la mémoire de l'IA (B1)`.

### Task 3: Exécution de la mémoire dans le moteur
**Files:** `mira-core/src/ai/memory/runtime.rs`, `runtime_local.rs`, `runtime_provider.rs`, `ai/state.rs` (champ `memory`), `api/engine.rs` (`shutdown_within`) ; tests `tests/ai_memory_runtime.rs`.
**Interfaces — Produces :** `MemoryRuntime::store(&self, engine: &Engine) -> Result<Arc<MemoryStore>, MiraError>` (construit selon les réglages, `incompatible_state.ai_model_missing` si le modèle local manque, refus si désactivée), `MemoryRuntime::reset(&self)` (jette le store), `MemoryRuntime::stop(&self)`. Mode `Provider` : adresse et clé de la connexion, modèle qualifié `<fournisseur>:<modèle>`.
- [ ] Tests avec le faux serveur : désactivée → erreur typée et aucun lancement ; local installé → vecteurs rendus et un seul lancement pour deux appels ; fournisseur → requête sur `/v1/embeddings` avec la clé ; `reset` après changement de réglage ; arrêt à la fermeture du moteur.
- [ ] Rouge, implémenter, vert, commit `feat(ia): exécution de la mémoire selon les réglages (B2, B3, B5)`.

### Task 4: Installation du modèle d'embeddings
**Files:** `mira-core/src/api/handlers/ai/memory_install.rs`, `ai/memory/install_plan.rs` ; tests `tests/ai_memory_install_op.rs`.
**Interfaces — Consumes :** `InstallPlan`-like de `ai/local/install.rs`, `install_embedding_model` (lot 1a). **Produces :** `handle_install_memory_model(engine) -> Result<Response, MiraError>` rendant une `Operation` ; progression sur une échelle (serveur puis modèle), annulable, partage `LocalInstallSlot`.
- [ ] Tests : opération qui installe serveur et modèle depuis l'hôte factice ; annulation ; refus si une installation est déjà en cours (`ai_local_busy`) ; sans `embedding` dans la fiche → `ai_local_unsupported_platform`.
- [ ] Rouge, implémenter, vert, commit `feat(ia): installation du modèle d'embeddings en arrière-plan (B4)`.

### Task 5: Opérations d'API
**Files:** `mira-core/src/api/contract/ai_requests.rs`, `ai_responses.rs`, `risk/assistant.rs`, `handlers/ai/mod.rs`, `handlers/ai/memory_*.rs`, `ai/memory/store_update.rs` (`MemoryStore::update`), `mira-protocol` golden ; tests `tests/api_ai_memory.rs`, `tests/api_ai_memory_privacy.rs`.
**Interfaces — Produces :** requêtes `GetMemoryStatus` (Read), `SetMemorySettings { settings }`, `InstallMemoryModel`, `ListMemories { limit, after }`, `SearchMemories { query, limit }`, `RememberMemory { text, kind }` (origine `user`), `UpdateMemory { id, text }`, `ForgetMemory { id }`, `ForgetAllMemories`, `ReindexMemories` ; réponses `MemoryStatus`, `Memories(Vec<MemoryRecord>)`, `MemoryHits(Vec<MemoryHit>)`, `Memory(MemoryRecord)`, `MemoryCount(u64)`, `MemoryReindexed { done, failed }`, `Done`.
- [ ] Tests : statut par défaut ; activer, retenir, lister, chercher, modifier, oublier, tout oublier ; secret refusé ; mémoire désactivée → erreur typée ; `SetMemorySettings` change d'embedder et `GetMemoryStatus.stale` > 0 puis `ReindexMemories` le ramène à 0 ; niveaux de risque (`Sensitive` sauf les lectures).
- [ ] Rouge, implémenter, vert (golden du protocole compris), commit `feat(ia): opérations d'API de la mémoire, contrat 1.11 (B9)`.

### Task 6: Outils du chat
**Files:** `mira-core/src/ai/tools/{recall_memory,remember,forget_memory}.rs`, `catalog.rs` (`specs_for(mode, memory_enabled)`, `kind_of`, `is_proposal`), `run.rs`, `propose.rs`, `run_result.rs` (`memory`), `ai/chat/{tool_exec,tool_step,progress,tools_plan}.rs` ; tests `tests/ai_tools_memory.rs`, `tests/ai_tools_loop_memory.rs`.
**Interfaces — Produces :** outils `recall_memory {query}` (lecture, `ContentKind::Memory`), `remember {text, kind}` et `forget_memory {id}` (propositions : `ToolRun::memory_proposed`, delta `MemoryProposed` avant `ToolFinished`). `specs_for` n'annonce ces outils qu'en mode Assistant et mémoire activée.
- [ ] Tests : outils absents si mémoire désactivée ou en mode composeur ; `recall_memory` rend les textes et identifiants ; `remember` valide (vide, trop long, secret → erreur) et n'écrit rien (liste vide après la boucle) ; `forget_memory` d'un identifiant inconnu → erreur, connu → carte avec le texte ; le delta arrive dans l'ordre.
- [ ] Rouge, implémenter, vert, commit `feat(ia): outils recall_memory, remember et forget_memory du chat (B7)`.

### Task 7: Rappel automatique et consentement
**Files:** `mira-core/src/ai/chat/memory_context.rs`, `prepare.rs`, `checks.rs` ; tests `tests/ai_chat_memory_context.rs`.
**Interfaces — Produces :** `memory_parts(engine, request) -> Vec<ContextPart>` (≤ 5 souvenirs, type `memory`) ajouté au contexte avant le rendu ; consentement par type `memory` pour une connexion non locale.
- [ ] Tests : mémoire activée → partie `memory` rendue et masquée ; désactivée ou composeur → aucune ; connexion cloud sans `memory` autorisé → refus typé existant ; échec d'embedder → discussion normale sans partie ; jamais plus de 5.
- [ ] Rouge, implémenter, vert, commit `feat(ia): rappel automatique des souvenirs dans le contexte (B6, B8)`.

---

## Lot 1c — application Flutter

### Task 8: Modèles et client
**Files:** `app/lib/engine/ai_memory.dart` (types et JSON), `ai_admin.dart` (appels), tests `app/test/ai_memory_models_test.dart`.
- [ ] Tests de JSON (réglages, statut, souvenir, proposition, delta `memory_proposed`) ; implémenter ; `flutter test` ciblé ; commit `feat(app): modèles et appels de la mémoire de l'IA`.

### Task 9: État et cartes de chat
**Files:** `app/lib/state/ai_memory_state.dart`, `chat_message.dart`, `chat_message_json.dart`, `chat_controller.dart`, `app/lib/ui/chat_memory_card.dart`, `assistant_panel.dart`, `chat_tool_labels.dart`, tests `app/test/chat_memory_*_test.dart`.
- [ ] Tests : delta → carte ; « Retenir » appelle `RememberMemory` puis marque la carte ; « Ignorer » n'écrit rien ; « Oublier » appelle `ForgetMemory` ; la carte survit à l'enregistrement de la conversation ; commit `feat(app): cartes Retenir et Oublier dans le chat`.

### Task 10: Réglages et liste des souvenirs
**Files:** `app/lib/ui/ai_memory_section.dart`, `ai_memory_list.dart`, `ai_memory_install.dart`, `ai_settings.dart`, `ai_kind_names.dart` (type `memory`), i18n `fr.dart`/`en.dart`, tests `app/test/ai_memory_*_test.dart`.
- [ ] Tests widget : interrupteur ; choix local avec progression et annulation ; choix fournisseur avec avertissement ; liste, recherche, modification, suppression ; « Tout oublier » avec confirmation ; « Réindexer » visible seulement si des souvenirs sont périmés ; commit `feat(app): réglages et liste des souvenirs`.

### Task 11: Vérification finale
- [ ] `tools/i18n-audit`, `flutter analyze`, `flutter test` des fichiers touchés, puis `./ci.sh`, puis relecture indépendante de la branche (Critical/Important corrigés par test d'abord), puis `docs/ARCHITECTURE-IMPLEMENTEE.md` et la spec mis à jour.

## Auto-revue

Couverture : M1→T2/T3/T10, M2→T3/T4, M4→T6/T9, M5→T7, M7→T5/T10, M9→T5, B6→T1/T7, B10→T3 (erreur non réessayable propagée). Types : `MemoryProposal`, `MemorySettings`, `MemoryStatus` définis en T1 et seuls utilisés ensuite. Risque : T3 dépend de la fiche `embedding` (T1) et de l'accès aux connexions pour le mode fournisseur.
