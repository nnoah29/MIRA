//! Composition root : seul bloc qui connaît le cœur et tous les providers (5.4).
//! Réexporte ce dont les clients (pont Flutter) ont besoin, pour qu'ils ne dépendent que d'ici.

pub mod ai_composition;
pub mod composition;
pub mod engine_config;
pub mod oauth_embedded;
pub mod oauth_file_load;
pub mod oauth_file_parse;
pub mod oauth_file_path;
pub mod oauth_overlay;

pub use ai_composition::build_ai_registry;
pub use composition::build_registry;
pub use engine_config::{production_config, production_config_with_oauth};
pub use mira_core::registry::{ProviderRegistry, resolve_account_provider};
pub use mira_core::{
    accounts, ai, api, auth, capabilities, fetching, html, mailboxes, operations, ports, profile,
    reading, registry, storage, sync,
};
pub use mira_vocabulary;
