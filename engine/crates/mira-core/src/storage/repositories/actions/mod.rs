//! Dépôt « actions » : file d'actions idempotente et ordonnée (voir docs/specs/J2-stockage.md).

mod enqueue;
mod payload;
mod read;
mod rows;
mod update;

pub use payload::ActionPayload;
