//! Fournisseur d'IA local (modèle sur la machine) : fiche statique, aucune entrée-sortie.

pub mod descriptor;
pub mod pinned;
mod pinned_embedding;
mod pinned_models;
mod pinned_server;
pub mod provider;

pub use descriptor::PROVIDER_ID;
pub use pinned_embedding::{embedding_model, embedding_server_args};
pub use provider::LocalAiProvider;
