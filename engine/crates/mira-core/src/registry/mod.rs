//! Registre des providers : enregistrement, détection, repli (6.6).

pub mod provider_registry;
pub mod resolve;

pub use provider_registry::ProviderRegistry;
pub use resolve::resolve_account_provider;
