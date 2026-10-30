//! Assistant d'IA enfichable : briques du cœur, sans connaître aucun fournisseur concret.

pub(crate) mod access;
pub(crate) mod base_url;
pub mod chat;
pub mod masking;
pub mod models;
pub mod registry;
pub mod sse;
pub mod tools;
pub mod transport;
pub mod wire;

pub use registry::AiProviderRegistry;
