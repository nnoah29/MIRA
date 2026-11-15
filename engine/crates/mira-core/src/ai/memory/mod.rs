//! Mémoire de l'IA : encodage des vecteurs, similarité et classement des souvenirs.

mod embedder;
mod embedding_install;
mod embeddings_wire;
mod http_embedder;
mod ranking;
mod vectors;

pub use embedder::{EmbedFuture, Embedder};
pub use embedding_install::{install_embedding_model, is_embedding_installed};
pub use embeddings_wire::{embeddings_request, parse_embeddings};
pub use http_embedder::HttpEmbedder;
pub use ranking::{cosine, top_k};
pub use vectors::{decode_vector, encode_vector};
