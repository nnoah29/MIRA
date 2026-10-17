//! Détection des changements d'une boîte selon les capacités du serveur (D-22) et repères (invariant 20).

mod changes;
mod detect;
mod dispatch;
mod finish;
mod first;
mod flag_changes;
mod headers;
mod plan;

pub use dispatch::sync_mailbox;
pub use finish::finish_mailbox;
pub use headers::{Announce, HEADER_BATCH, ingest_batch, ingest_headers, newest_first};
