//! Formats de dialogue avec les services d'IA : requête, flux et liste de modèles.

pub mod call;
pub mod openai_decode;
pub mod openai_encode;
pub mod openai_models;

pub use call::HttpCall;
