//! Liste des modèles d'une connexion d'IA : relevé auprès du service, filtre, tri et cache.

mod fetch;
mod filter;
mod sort;

pub use fetch::fetch_models;
pub use filter::filter_models;
pub use sort::sort_models;
