# Mémoire de l'IA, lot 1a (stockage, embeddings locaux, téléchargement) — plan d'implémentation

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal :** poser le socle de la mémoire de l'IA : table de souvenirs avec vecteurs, recherche par similarité, interface d'embedder avec implémentation locale (serveur llama.cpp dédié), et téléchargement vérifié du modèle d'embeddings. Aucune exposition au chat ni à l'API dans ce lot.

**Architecture :** un module `mira-core/src/ai/memory/` (similarité pure, interface `Embedder`, service `MemoryStore`) s'appuie sur un dépôt SQLite `ai_memories` (méthodes de `Tx`) et sur les briques `ai/local/` existantes (téléchargement épinglé, gestionnaire de serveur). Le modèle d'embeddings est épinglé dans `mira-ai-local` ; le cœur ne nomme aucun fournisseur.

**Tech Stack :** Rust 1.99 / édition 2024, `rusqlite` (BLOB `f32` little-endian), `tokio`, llama.cpp (`--embedding`, route `/v1/embeddings`).

**Spec :** `docs/superpowers/specs/2026-11-15-ia-memoire-design.md` (décisions M1–M3, M6, M8 ; lot 1a du §6).

## Global Constraints

- Zéro `unsafe`, zéro `unwrap`/`expect`/`panic!` hors tests ; fonction ≤ 30 lignes ; fichier ≤ 6 fonctions avec corps, tests dans des fichiers séparés.
- Aucun commentaire dans un corps de fonction ; une ligne `///` en français avant chaque `fn` ; `//!` de 1 à 3 lignes en tête de fichier.
- Pas de `utils`/`helpers`/`common`/`misc` ; identifiants en anglais, textes en français.
- `mira-core` ne nomme aucun fournisseur ; `mira-ai-local` dépend seulement de `mira-vocabulary`.
- Tests uniquement contre `127.0.0.1` ; aucun secret dans erreurs, journaux ni souvenirs.
- Tests ciblés (`cargo test -p mira-core --test <fichier>`, `CARGO_BUILD_JOBS=4`) ; la suite complète et `./ci.sh` ne tournent pas sans accord de l'utilisateur.
- Travail dans le worktree `../MIRA-memoire` (branche `feat/ia-memoire`) ; un commit par tâche, message `type(domaine): résumé`, sans trailer d'attribution.

## Review Focus

- Deux souvenirs de **modèles d'embeddings différents** ne se comparent jamais : une recherche ignore les vecteurs `stale` (test T6).
- Un vecteur de **dimension inattendue** (modèle changé, serveur défaillant) est rejeté sans corrompre la base (T5, T6).
- Un souvenir **vide, blanc ou énorme** (> 2 000 caractères) est refusé avec une erreur typée (T6).
- Un texte contenant un **secret détecté** n'est jamais stocké (T6, M8).
- **Téléchargement coupé** puis repris : le fichier final n'est installé qu'après contrôle de taille et SHA-256 (T4, mécanisme existant réutilisé).

---

### Task 1: Types du vocabulaire

**Files:**
- Create: `engine/crates/mira-vocabulary/src/ai/memory.rs`
- Modify: `engine/crates/mira-vocabulary/src/ai/mod.rs` (déclarer et ré-exporter)
- Test: `engine/crates/mira-vocabulary/tests/ai_memory.rs`

**Interfaces:**
- Produces : `MemoryKind { Fact, Preference, Action }`, `MemorySource { User, Ai }` (serde `snake_case`), `MemoryRecord { id: String, text: String, kind: MemoryKind, source: MemorySource, created_at: i64, model_id: Option<String> }` (sans le vecteur), `MemoryHit { record: MemoryRecord, score: f32 }`.

- [ ] **Step 1 : test** `memory_kind_serializes_snake_case` : `serde_json::to_string(&MemoryKind::Preference)` vaut `"\"preference\""` ; `memory_record_round_trips` : sérialiser puis désérialiser un `MemoryRecord` rend une valeur égale.
- [ ] **Step 2 :** `cargo test -p mira-vocabulary --test ai_memory` → échec (types absents).
- [ ] **Step 3 :** définir les quatre types (`Debug, Clone, PartialEq, Serialize, Deserialize` ; `Eq` seulement sur les énumérations).
- [ ] **Step 4 :** relancer → PASS.
- [ ] **Step 5 :** `git commit -m "feat(ia): types de la mémoire dans le vocabulaire"`.

### Task 2: Migration et dépôt `ai_memories`

**Files:**
- Create: `engine/crates/mira-core/src/storage/migrations/v021_ai_memories.sql`, `engine/crates/mira-core/src/storage/repositories/ai_memories/{mod,read,write,rows}.rs`
- Modify: `storage/migrations/mod.rs` (ajouter `include_str!("v021_ai_memories.sql")` après v020), `storage/repositories/mod.rs` (`pub mod ai_memories;`), `storage/erasure.rs` (l'effacement du profil vide aussi la table : lire le fichier et suivre son mécanisme)
- Test: `engine/crates/mira-core/tests/storage_ai_memories.rs`

**Interfaces:**
- Consumes : `MemoryRecord`, `MemoryKind`, `MemorySource` (T1) ; `Tx::sql()`, `storage_error`.
- Produces (méthodes de `Tx`) :
  `insert_ai_memory(&self, record: &MemoryRecord, embedding: &[u8], dim: u32) -> Result<(), MiraError>` ;
  `list_ai_memories(&self, limit: u32, after: Option<&str>) -> Result<Vec<MemoryRecord>, MiraError>` ;
  `ai_memory_vectors(&self, model_id: &str) -> Result<Vec<(MemoryRecord, Vec<u8>, u32)>, MiraError>` (hors `stale`, modèle donné) ;
  `update_ai_memory_text(&self, id: &str, text: &str, embedding: &[u8], dim: u32, model_id: &str) -> Result<(), MiraError>` ;
  `delete_ai_memory(&self, id: &str) -> Result<(), MiraError>` (inconnu : `not_found.ai_memory`) ;
  `delete_all_ai_memories(&self) -> Result<u64, MiraError>` ;
  `mark_ai_memories_stale_except(&self, model_id: &str) -> Result<u64, MiraError>`.
- Table : `ai_memories(id TEXT PRIMARY KEY, text TEXT NOT NULL, kind TEXT NOT NULL, source TEXT NOT NULL, created_at INTEGER NOT NULL, model_id TEXT, embedding BLOB, dim INTEGER, stale INTEGER NOT NULL DEFAULT 0)` + index sur `created_at`.

- [ ] **Step 1 : tests** (style `storage_ai_conversations.rs`) : `insert_then_list_returns_newest_first` ; `vectors_exclude_stale_and_other_models` (deux souvenirs de modèles `m1`/`m2`, `mark_ai_memories_stale_except("m2")` ; `ai_memory_vectors("m2")` ne rend que le second) ; `delete_unknown_is_not_found` ; `delete_all_returns_count` ; `profile_erasure_clears_memories`.
- [ ] **Step 2 :** `cargo test -p mira-core --test storage_ai_memories` → échec.
- [ ] **Step 3 :** écrire migration et méthodes (SQL paramétré comme `ai_conversations/write.rs`) ; brancher l'effacement.
- [ ] **Step 4 :** relancer → PASS ; lancer aussi `--test storage_ai_conversations` pour vérifier que les migrations passent toujours.
- [ ] **Step 5 :** `git commit -m "feat(stockage): dépôt des souvenirs de l'IA (D-27, M3, M7)"`.

### Task 3: Similarité (module pur)

**Files:**
- Create: `engine/crates/mira-core/src/ai/memory/{mod,vectors,ranking}.rs`
- Modify: `engine/crates/mira-core/src/ai/mod.rs` (`pub(crate) mod memory;`)
- Test: `engine/crates/mira-core/tests/ai_memory_ranking.rs` (via un `pub` ré-exporté sous `#[doc(hidden)]` si les tests d'intégration en ont besoin, comme le font les autres modules du crate)

**Interfaces:**
- Produces :
  `encode_vector(values: &[f32]) -> Vec<u8>` et `decode_vector(bytes: &[u8], dim: u32) -> Option<Vec<f32>>` (`None` si la longueur ne vaut pas `4 × dim`) ;
  `cosine(a: &[f32], b: &[f32]) -> Option<f32>` (`None` si dimensions différentes ou norme nulle) ;
  `top_k(query: &[f32], items: Vec<(String, Vec<f32>)>, k: usize, min_score: f32) -> Vec<(String, f32)>` (décroissant, ex æquo départagés par identifiant).

- [ ] **Step 1 : tests** : `cosine_of_identical_vectors_is_one` ; `cosine_of_orthogonal_vectors_is_zero` ; `cosine_rejects_dimension_mismatch` (`None`) ; `cosine_rejects_zero_norm` ; `decode_rejects_wrong_length` ; `encode_decode_round_trip` ; `top_k_orders_and_cuts` (k = 2, seuil 0,5) ; `top_k_ties_break_by_id`.
- [ ] **Step 2 :** échec attendu (fonctions absentes).
- [ ] **Step 3 :** implémenter dans `vectors.rs` (encodage `f32::to_le_bytes`) et `ranking.rs`.
- [ ] **Step 4 :** PASS.
- [ ] **Step 5 :** `git commit -m "feat(ia): similarité cosinus et classement des souvenirs"`.

### Task 4: Modèle d'embeddings épinglé et téléchargement

**Files:**
- Create: `engine/crates/mira-ai-local/src/pinned_embedding.rs`, `engine/crates/mira-core/src/ai/memory/embedding_install.rs`
- Modify: `mira-ai-local/src/lib.rs` (exposer `embedding_model() -> AiLocalModel`) ; `mira-core/src/ai/local/paths.rs` (ajouter `LocalPaths::embedding_file(&self, model: &AiLocalModel) -> PathBuf` sous `local-ai/embeddings/`) ; `docs/SOURCES.md` (source du modèle)
- Test: `engine/crates/mira-ai-local/tests/embedding_model.rs`, `engine/crates/mira-core/tests/ai_memory_embedding_install.rs`

**Interfaces:**
- Consumes : `AiLocalModel`, `download_file`, `DownloadJob`, `CancelFlag` (`ai/local/download.rs`), `LocalPaths`.
- Produces : `embedding_model() -> AiLocalModel` (id, URL d'hébergement, taille, SHA-256 épinglés) ; `install_embedding_model(paths: &LocalPaths, model: &AiLocalModel, resources: &dyn SystemResources, progress: &(dyn Fn(u64, u64) + Send + Sync), cancel: &CancelFlag) -> Result<PathBuf, MiraError>` ; `is_embedding_installed(paths: &LocalPaths, model: &AiLocalModel) -> bool`.

- [ ] **Step 1 : choisir et épingler le modèle.** Un agent `architecte` (web autorisé, contenu web = donnée) propose un GGUF d'embeddings multilingue ≤ 300 Mo, licence permissive ; l'URL de téléchargement (`.../resolve/main/<fichier>`), la taille et le SHA-256 sont relevés sur la page du dépôt puis **recalculés** après un vrai téléchargement local (`sha256sum`) avant d'être écrits dans le code. Source consignée dans `docs/SOURCES.md`.
- [ ] **Step 2 : tests** `mira-ai-local` : `embedding_model_is_pinned` (id non vide, SHA-256 de 64 caractères hexadécimaux, taille > 0, URL en `https://`) ; `mira-core` : `install_downloads_from_fake_server_and_verifies` (serveur factice sur `127.0.0.1` comme `ai_local_download.rs`), `install_rejects_wrong_sha256` (aucun fichier final), `is_installed_false_before_true_after`.
- [ ] **Step 3 :** échec attendu.
- [ ] **Step 4 :** implémenter ; `install_embedding_model` appelle `download_file` avec `LocalPaths::embedding_file` comme destination.
- [ ] **Step 5 :** PASS ; `cargo test -p mira-ai-local`.
- [ ] **Step 6 :** `git commit -m "feat(ia): modèle d'embeddings épinglé et installation vérifiée (M2)"`.

### Task 5: Interface `Embedder` et implémentation locale

**Files:**
- Create: `engine/crates/mira-core/src/ai/memory/{embedder,local_embedder,embeddings_wire}.rs`
- Modify: `engine/crates/mira-ai-local/src/pinned.rs` (ajouter à `AiLocalRuntime` le gabarit d'arguments d'embeddings : `--embedding`, sans contexte de chat) et `mira-vocabulary/src/ai/local_runtime.rs` (champ `embedding_args: Vec<String>`) ; `ai/local/runtime.rs` si le gestionnaire doit accepter un second serveur
- Test: `engine/crates/mira-core/tests/ai_memory_embedder.rs`

**Interfaces:**
- Consumes : `LocalRuntimeManager::ensure_running`, `LocalRuntimeHandle`, `build_args` ; client HTTP de `ai/transport/`.
- Produces :
  `#[async_trait] trait Embedder: Send + Sync { fn model_id(&self) -> &str; async fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, MiraError>; }` ;
  `LocalEmbedder::new(handle_provider, model: AiLocalModel)` implémentant `Embedder` en appelant `POST /v1/embeddings` (`{"model": id, "input": [...]}`, réponse OpenAI `data[i].embedding`).
- Décision : serveur **dédié** aux embeddings (second `LocalRuntimeManager`), arrêté par inactivité comme le serveur de chat (`idle_stop_seconds`) ; le modèle de chat n'est pas affecté.

- [ ] **Step 1 : tests** contre un serveur factice `127.0.0.1` : `embed_returns_one_vector_per_text` ; `embed_rejects_inconsistent_dimensions` (erreur typée `incompatible_state`) ; `embed_maps_server_error_without_leaking_key` (message sans clé d'API) ; `embed_empty_input_makes_no_request`.
- [ ] **Step 2 :** échec attendu.
- [ ] **Step 3 :** implémenter la requête et l'analyse de réponse (`embeddings_wire.rs`), la clé d'API venant du fichier de clé existant (`key_file.rs`).
- [ ] **Step 4 :** PASS ; relancer `--test ai_local_args` (le gabarit change).
- [ ] **Step 5 :** `git commit -m "feat(ia): interface Embedder et embedder local llama.cpp (M2, M6)"`.

### Task 6: Service `MemoryStore`

**Files:**
- Create: `engine/crates/mira-core/src/ai/memory/{store,store_search,guard}.rs`
- Test: `engine/crates/mira-core/tests/ai_memory_store.rs` (avec un `Embedder` factice déterministe défini dans `tests/support_ai_memory/`)

**Interfaces:**
- Consumes : T1 (types), T2 (méthodes de `Tx`), T3 (`cosine`, `top_k`, encodage), T5 (`Embedder`), masquage `ai/masking` (M8).
- Produces :
  `MemoryStore::new(engine_storage, embedder: Arc<dyn Embedder>, clock)` ;
  `async fn remember(&self, text: &str, kind: MemoryKind, source: MemorySource) -> Result<MemoryRecord, MiraError>` ;
  `async fn recall(&self, query: &str, k: usize) -> Result<Vec<MemoryHit>, MiraError>` ;
  `async fn forget(&self, id: &str) -> Result<(), MiraError>` ;
  `async fn forget_all(&self) -> Result<u64, MiraError>` ;
  `async fn reindex(&self) -> Result<u64, MiraError>` (marque `stale` les vecteurs d'un autre modèle puis les recalcule par lots de 16).
- Constantes : texte ≤ 2 000 caractères, `min_score` 0,3, `k` plafonné à 10 (valeurs initiales, ajustables, consignées dans `DECISIONS-AGENT.md`).

- [ ] **Step 1 : tests** : `remember_then_recall_finds_it` ; `recall_ignores_stale_vectors_of_other_model` ; `remember_rejects_blank_text` ; `remember_rejects_text_over_limit` ; `remember_rejects_secret_like_text` (clé du type `sk-…` : refus, rien en base) ; `embedder_failure_stores_nothing` ; `forget_removes_from_recall` ; `reindex_recomputes_stale_with_new_model` ; `recall_with_no_memories_is_empty_without_calling_embedder`.
- [ ] **Step 2 :** échec attendu.
- [ ] **Step 3 :** implémenter (validation et filtre dans `guard.rs`, recherche dans `store_search.rs`) ; l'écriture se fait dans une transaction après l'appel à l'embedder, jamais pendant.
- [ ] **Step 4 :** PASS ; relancer `ai_memory_ranking` et `storage_ai_memories`.
- [ ] **Step 5 :** vérifier les normes : `tools/check-norms` et `tools/check-architecture` (ciblés, sans `./ci.sh`), `cargo clippy -p mira-core -p mira-ai-local -p mira-vocabulary --all-targets -- -D warnings`, `cargo fmt --check`.
- [ ] **Step 6 :** mettre à jour `docs/dev/DECISIONS-AGENT.md` (valeurs de k, seuil, serveur dédié), puis `git commit -m "feat(ia): service MemoryStore (souvenir, rappel, oubli, ré-indexation)"`.

---

## Auto-revue

- **Couverture :** M1 et M4/M5/M7/M9 relèvent des lots 1b/1c (hors périmètre, aucune exposition ici) ; M2 → T4, M3 → T1/T2, M6 → T2/T6, M8 → T6.
- **Cohérence des types :** `MemoryRecord` (T1) est celui des méthodes de `Tx` (T2) et du service (T6) ; `Embedder::embed` (T5) est consommé tel quel par T6 ; `cosine`/`top_k` (T3) par T6.
- **Risque ouvert :** gestion d'un second serveur llama.cpp (T5) selon que `LocalRuntimeManager` accepte une deuxième instance ; si non, l'adapter est une sous-tâche de T5 à consigner.
- **Proportion :** plan court, aucun corps de fonction recopié.
