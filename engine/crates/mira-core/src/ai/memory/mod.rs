//! Mémoire de l'IA : encodage des vecteurs, similarité et classement des souvenirs.

mod current_model;
mod embedder;
mod embedding_install;
mod embeddings_wire;
mod engine_memory;
mod forgetting;
mod guard;
mod http_embedder;
mod local_embedder;
mod memory_install;
mod provider_embedder;
mod ranking;
mod runtime;
mod store;
mod store_reindex;
mod store_search;
mod store_update;
mod vectors;

pub(crate) use current_model::current_model_id;
pub use embedder::{EmbedFuture, Embedder};
pub use embedding_install::{install_embedding_model, is_embedding_installed};
pub use embeddings_wire::{embeddings_request, parse_embeddings};
pub(crate) use forgetting::{forget_everything, forget_one};
pub use http_embedder::HttpEmbedder;
pub use local_embedder::{LocalEmbedder, LocalEmbedderSetup, embedding_runtime};
pub(crate) use memory_install::{MemoryInstallPlan, run_memory_install};
pub use ranking::{cosine, top_k};
pub(crate) use runtime::MemoryRuntime;
pub use store::MemoryStore;
pub use store_reindex::ReindexReport;
pub use vectors::{decode_vector, encode_vector};
