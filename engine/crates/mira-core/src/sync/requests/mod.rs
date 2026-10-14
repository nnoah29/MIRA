//! Demandes de synchronisation adressées à la tâche d'un compte : fusionnées, avec accusé de fin.

mod queue;
mod scope;
mod taken;

pub use queue::{SyncRequests, Ticket};
pub use scope::RequestScope;
pub use taken::TakenRequests;
