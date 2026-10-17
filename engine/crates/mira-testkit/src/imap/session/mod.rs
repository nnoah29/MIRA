//! Session IMAP : une connexion, son état et sa boucle de commandes.

pub(crate) mod flow;
pub(crate) mod idle;
pub(crate) mod literal_marker;
pub(crate) mod log;
pub(crate) mod reader;
pub(crate) mod run;
pub(crate) mod state;
pub(crate) mod sync;
pub(crate) mod sync_announce;
pub(crate) mod transport;
pub(crate) mod writable;
