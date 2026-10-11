//! Serveurs fixes du service, identiques pour toutes ses adresses.

use mira_vocabulary::{Security, ServerEndpoint, ServerSettings};

/// Construit un point d'accès avec son mode de chiffrement.
fn endpoint(host: &str, port: u16, security: Security) -> ServerEndpoint {
    ServerEndpoint {
        host: host.to_string(),
        port,
        security,
    }
}

/// Donne l'IMAP `outlook.office365.com:993` en TLS et le SMTP `smtp.office365.com:587` en STARTTLS.
pub fn servers() -> ServerSettings {
    ServerSettings {
        imap: endpoint("outlook.office365.com", 993, Security::Tls),
        smtp: endpoint("smtp.office365.com", 587, Security::StartTls),
    }
}
