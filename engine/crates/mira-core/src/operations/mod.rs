//! Tâches longues (12.2.4) : identifiant d'opération, progression par événements, annulation coopérative.

mod handle;
mod registry;
mod status;

pub use handle::OperationHandle;
pub use registry::OperationRegistry;
pub use status::{OperationInfo, OperationStatus};
