//! Assistant d'IA enfichable : briques du cœur, sans connaître aucun fournisseur concret.

pub(crate) mod access;
pub(crate) mod base_url;
pub mod chat;
pub mod key_login;
pub mod local;
pub mod masking;
pub mod models;
pub mod registry;
pub(crate) mod session;
pub mod sse;
pub(crate) mod state;
pub mod tools;
pub mod transport;
pub mod wire;

pub use registry::AiProviderRegistry;
