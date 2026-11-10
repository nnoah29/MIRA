//! Fournisseur d'IA local (modèle sur la machine) : fiche statique, aucune entrée-sortie.

pub mod descriptor;
pub mod pinned;
pub mod provider;

pub use descriptor::PROVIDER_ID;
pub use provider::LocalAiProvider;
