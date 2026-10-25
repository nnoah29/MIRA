//! Dépôt « message_templates » : modèles de messages communs à l'app et leurs clés d'idempotence.

mod errors;
mod idempotency;
mod read;
mod rows;
mod write;

pub use idempotency::TemplateIdempotencyEntry;
