//! Composition root : seul bloc qui connaît le cœur et tous les providers (5.4).
//! Réexporte ce dont les clients (CLI) ont besoin, pour qu'ils ne dépendent que d'ici.

pub mod composition;

pub use composition::build_registry;
pub use mira_core::registry::{ProviderRegistry, resolve_account_provider};
pub use mira_core::{capabilities, mailboxes, registry};
pub use mira_vocabulary;
