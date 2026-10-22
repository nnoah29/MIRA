//! Dépôt « rules » : règles ordonnées, portée par compte et file des messages à évaluer.

mod candidates;
mod idempotency;
mod order;
mod pending;
mod read;
mod rows;
mod write;

pub use idempotency::RuleIdempotencyEntry;
