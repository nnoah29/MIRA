//! Mémoire de l'IA : encodage des vecteurs, similarité et classement des souvenirs.

mod ranking;
mod vectors;

pub use ranking::{cosine, top_k};
pub use vectors::{decode_vector, encode_vector};
