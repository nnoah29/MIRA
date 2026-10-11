//! Cibles serveur d'une action : relevé à la soumission, relecture au rejeu.

mod capture;
mod load;
mod model;

pub(crate) use capture::capture_server_targets;
pub use load::load_server_targets;
pub use model::{AppearanceTarget, MailboxTarget, MessageTarget, ServerTargets};
