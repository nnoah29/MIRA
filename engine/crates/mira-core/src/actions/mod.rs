//! File d'actions locale : soumission idempotente, effet local optimiste, annulation, relance, rebase (D-23).
//! Le rejeu vers le serveur n'en fait pas partie : il lit les cibles serveur et l'état de la file.

mod cancel;
mod effect;
pub(crate) mod errors;
mod model;
mod plan;
mod queue;
mod rebase;
mod record;
mod resolve;
mod retry;
mod revert;
mod steps;
mod submit;
mod targets;

pub use cancel::cancel;
pub use model::{
    LocalIntent, LocalStep, QueueContext, SEND_UNDO_DELAY_MILLIS, SubmitRequest, UNDO_DELAY_MILLIS,
    UndoRecord,
};
pub(crate) use queue::{Queued, queue_action};
pub use rebase::reapply_pending_local_effects;
pub use retry::retry;
pub use revert::revert_local_effect;
pub use submit::submit;
pub use targets::{
    AppearanceTarget, MailboxTarget, MessageTarget, ServerTargets, load_server_targets,
};
