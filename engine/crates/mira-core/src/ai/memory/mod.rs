//! Mémoire de l'IA : encodage des vecteurs, similarité et classement des souvenirs.

mod embedding_install;
mod ranking;
mod vectors;

pub use embedding_install::{install_embedding_model, is_embedding_installed};
pub use ranking::{cosine, top_k};
pub use vectors::{decode_vector, encode_vector};
