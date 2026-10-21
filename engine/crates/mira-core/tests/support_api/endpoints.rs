//! Adresses des faux serveurs de test : IMAP et SMTP sur la boucle locale, domaines de test redirigés.

use mira_core::api::EngineConfig;
use mira_vocabulary::{Security, ServerEndpoint, ServerSettings};

/// Serveurs en clair sur la boucle locale
pub fn local_servers(imap_port: u16, smtp_port: u16) -> ServerSettings {
    let endpoint = |port: u16| ServerEndpoint {
        host: "127.0.0.1".to_string(),
        port,
        security: Security::Plaintext,
    };
    ServerSettings {
        imap: endpoint(imap_port),
        smtp: endpoint(smtp_port),
    }
}

/// Redirige les domaines de test vers les faux serveurs IMAP et SMTP
pub fn redirect_domains(config: &mut EngineConfig, imap_port: u16, smtp_port: u16) {
    for domain in ["folders.test", "labels.test", "oauth.test"] {
        let servers = local_servers(imap_port, smtp_port);
        config
            .network
            .server_overrides
            .insert(domain.to_string(), servers);
    }
}
