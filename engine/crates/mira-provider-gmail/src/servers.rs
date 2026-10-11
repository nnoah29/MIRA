//! Serveurs fixes du service, identiques pour toutes ses adresses.

use mira_vocabulary::{Security, ServerEndpoint, ServerSettings};

/// Construit un point d'accès chiffré par TLS implicite.
fn tls_endpoint(host: &str, port: u16) -> ServerEndpoint {
    ServerEndpoint {
        host: host.to_string(),
        port,
        security: Security::Tls,
    }
}

/// Donne l'IMAP `imap.gmail.com:993` et le SMTP `smtp.gmail.com:465`, tous deux en TLS.
pub fn servers() -> ServerSettings {
    ServerSettings {
        imap: tls_endpoint("imap.gmail.com", 993),
        smtp: tls_endpoint("smtp.gmail.com", 465),
    }
}
