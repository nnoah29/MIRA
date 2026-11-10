//! Fournisseur d'IA local (modèle sur la machine) : fiche statique, aucune entrée-sortie.

pub mod descriptor;
pub mod pinned;
mod pinned_models;
mod pinned_server;
pub mod provider;

pub use descriptor::PROVIDER_ID;
pub use provider::LocalAiProvider;
