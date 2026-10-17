//! Serveur TCP : écoute, suivi des connexions, poignée de contrôle.

pub(crate) mod accept;
pub(crate) mod context;
mod control;
mod handle;
mod start;
pub(crate) mod tracker;

pub use handle::FakeImapServer;
