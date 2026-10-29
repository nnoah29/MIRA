//! Formats de dialogue avec les services d'IA : requête, flux et liste de modèles.

pub mod anthropic_decode;
pub mod anthropic_encode;
pub mod anthropic_models;
mod anthropic_usage;
pub mod call;
pub mod dispatch;
pub mod openai_decode;
pub mod openai_encode;
pub mod openai_models;
pub mod rfc3339;

pub use call::HttpCall;
