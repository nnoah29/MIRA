//! Identité d'un message (D-13, P7) : reconnaître, créer, placer dans les boîtes, drapeaux du serveur.

mod create;
mod flags;
mod incoming;
mod ingest;
mod placement;
mod recognize;
mod remember;
mod summary;

pub use flags::{is_deleted, server_flags};
pub use incoming::Incoming;
pub use ingest::{Ingested, apply_server_flags, ingest_header};
pub use summary::address_text;
