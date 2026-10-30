//! Formats de dialogue avec les services d'IA : requête, flux et liste de modèles.

pub mod anthropic_decode;
pub mod anthropic_encode;
pub mod anthropic_models;
mod anthropic_tool_decode;
mod anthropic_tools;
mod anthropic_usage;
pub mod call;
pub mod dispatch;
mod json_text;
pub mod openai_decode;
pub mod openai_encode;
pub mod openai_models;
mod openai_tool_decode;
mod openai_tools;
pub mod rfc3339;

pub use call::HttpCall;
