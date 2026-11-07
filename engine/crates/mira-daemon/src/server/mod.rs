//! Serveur : contexte partagé par toutes les connexions et boucle d'acceptation d'un écouteur.

mod accept;
mod clients;
mod closing;
mod context;

pub use accept::serve;
pub use clients::{ClientGuard, Clients};
pub use closing::wait_closing;
pub use context::ServerContext;
