//! Contrat des écritures : actions sur les messages et les boîtes, rédaction et envoi, file d'actions.

mod action_requests;
mod compose_requests;
mod draft_changes;
mod queue_requests;
mod target_selection;
mod write_response;

pub use action_requests::ActionRequest;
pub use compose_requests::{ComposeRequest, DraftMode};
pub use draft_changes::DraftChanges;
pub use queue_requests::QueueRequest;
pub use target_selection::TargetSelection;
pub use write_response::{OutboxEntry, WriteResponse};
