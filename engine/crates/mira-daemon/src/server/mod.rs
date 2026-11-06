//! Serveur : contexte partagé par toutes les connexions et boucle d'acceptation d'un écouteur.

mod accept;
mod closing;
mod context;

pub use accept::serve;
pub use closing::wait_closing;
pub use context::ServerContext;
