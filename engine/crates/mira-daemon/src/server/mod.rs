//! Serveur : contexte partagé par toutes les connexions et boucle d'acceptation d'un écouteur.

mod accept;
mod context;

pub use accept::serve;
pub use context::{ServerContext, wait_closing};
