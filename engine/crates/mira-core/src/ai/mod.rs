//! Assistant d'IA enfichable : briques du cœur, sans connaître aucun fournisseur concret.

pub mod models;
pub mod registry;
pub mod sse;
pub mod transport;
pub mod wire;

pub use registry::AiProviderRegistry;
