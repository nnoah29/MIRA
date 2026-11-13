//! Fournisseur d'IA ChatGPT (connexion de l'abonné) : fiche statique, aucune entrée-sortie.

pub mod descriptor;
pub mod provider;

pub use descriptor::PROVIDER_ID;
pub use provider::ChatGptAiProvider;
