//! Composition root : seul bloc qui connaît le cœur et tous les providers (5.4).
//! Réexporte ce dont les clients (CLI) ont besoin, pour qu'ils ne dépendent que d'ici.

pub mod composition;
pub mod engine_config;

pub use composition::build_registry;
pub use engine_config::production_config;
pub use mira_core::registry::{ProviderRegistry, resolve_account_provider};
pub use mira_core::{
    accounts, api, auth, capabilities, fetching, html, mailboxes, operations, ports, profile,
    reading, registry, storage, sync,
};
pub use mira_vocabulary;
