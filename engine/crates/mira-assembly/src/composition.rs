//! Construction du registre des providers : un seul endroit qui nomme les services (5.4).

use std::sync::Arc;

use mira_core::registry::ProviderRegistry;
use mira_provider_generic::GenericProvider;
use mira_provider_gmail::GmailProvider;
use mira_provider_outlook::OutlookProvider;
use mira_provider_yahoo::YahooProvider;

/// Construit le registre : repli générique, puis gmail, outlook et yahoo dans cet ordre.
pub fn build_registry() -> ProviderRegistry {
    let mut registry = ProviderRegistry::new(Arc::new(GenericProvider));
    registry.register(Arc::new(GmailProvider));
    registry.register(Arc::new(OutlookProvider));
    registry.register(Arc::new(YahooProvider));
    registry
}
