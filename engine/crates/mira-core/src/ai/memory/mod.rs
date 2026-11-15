//! Mémoire de l'IA : encodage des vecteurs, similarité et classement des souvenirs.

mod embedder;
mod embedding_install;
mod embeddings_wire;
mod guard;
mod http_embedder;
mod local_embedder;
mod ranking;
mod store;
mod store_reindex;
mod store_search;
mod vectors;

pub use embedder::{EmbedFuture, Embedder};
pub use embedding_install::{install_embedding_model, is_embedding_installed};
pub use embeddings_wire::{embeddings_request, parse_embeddings};
pub use http_embedder::HttpEmbedder;
pub use local_embedder::{LocalEmbedder, LocalEmbedderSetup, embedding_runtime};
pub use ranking::{cosine, top_k};
pub use store::MemoryStore;
pub use vectors::{decode_vector, encode_vector};
