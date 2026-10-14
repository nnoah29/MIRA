//! Configuration du moteur de test : trousseau et horloge gardés par le test, domaines redirigés vers les faux serveurs.

use std::path::Path;
use std::sync::Arc;

use mira_core::api::EngineConfig;
use mira_core::ports::{ManualClock, MemorySecretStore};
use mira_vocabulary::{Security, ServerEndpoint, ServerSettings};

use crate::support_api::config_with_secrets;

/// Point d'accès en clair sur la boucle locale
pub fn endpoint(port: u16) -> ServerEndpoint {
    ServerEndpoint {
        host: "127.0.0.1".to_string(),
        port,
        security: Security::Plaintext,
    }
}

/// Serveurs IMAP et SMTP de test pour ces ports
pub fn servers(imap_port: u16, smtp_port: u16) -> ServerSettings {
    ServerSettings {
        imap: endpoint(imap_port),
        smtp: endpoint(smtp_port),
    }
}

/// Configuration dont `domain` mène aux faux serveurs ; l'horloge et le trousseau sont ceux du test
pub fn build_config(
    home: &Path,
    secrets: &Arc<MemorySecretStore>,
    clock: &Arc<ManualClock>,
    domain: &str,
    servers: ServerSettings,
) -> EngineConfig {
    let mut config = config_with_secrets(home, Arc::clone(secrets));
    config.clock = Arc::clone(clock) as Arc<dyn mira_core::ports::Clock>;
    let overrides = &mut config.network.server_overrides;
    overrides.insert("other.test".to_string(), servers.clone());
    overrides.insert(domain.to_string(), servers);
    config
}
