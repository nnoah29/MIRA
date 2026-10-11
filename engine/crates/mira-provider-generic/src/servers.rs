//! Serveurs conventionnels déduits du domaine de l'adresse.

use mira_vocabulary::{Security, ServerEndpoint, ServerSettings};

/// Port IMAP conventionnel avec TLS implicite.
pub const IMAP_PORT: u16 = 993;
/// Port SMTP conventionnel avec TLS implicite.
pub const SMTP_PORT: u16 = 465;

/// Construit un point d'accès chiffré par TLS implicite.
fn tls_endpoint(host: String, port: u16) -> ServerEndpoint {
    ServerEndpoint {
        host,
        port,
        security: Security::Tls,
    }
}

/// Déduit `imap.<domaine>` et `smtp.<domaine>` en TLS implicite.
pub fn servers_for_domain(domain: &str) -> ServerSettings {
    ServerSettings {
        imap: tls_endpoint(format!("imap.{domain}"), IMAP_PORT),
        smtp: tls_endpoint(format!("smtp.{domain}"), SMTP_PORT),
    }
}
