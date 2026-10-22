//! Dépôt « local_labels » : libellés locaux communs aux comptes et leur pose par conversation.

mod assignments;
mod idempotency;
mod read;
mod rows;
mod write;

pub use idempotency::LabelIdempotencyEntry;
pub use read::require_label;
