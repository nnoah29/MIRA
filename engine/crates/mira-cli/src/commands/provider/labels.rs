//! Libellés lisibles des données d'un provider pour la sortie humaine.

use mira_assembly::mira_vocabulary::{AuthMethod, Security, ServerEndpoint};

/// Nom court d'une méthode d'authentification.
pub fn auth_label(method: &AuthMethod) -> &'static str {
    match method {
        AuthMethod::OAuth2(_) => "oauth2",
        AuthMethod::AppPassword => "app_password",
        AuthMethod::Password => "password",
    }
}

/// Nom court d'un mode de chiffrement.
pub fn security_label(security: Security) -> &'static str {
    match security {
        Security::Tls => "tls",
        Security::StartTls => "starttls",
        Security::Plaintext => "plaintext",
    }
}

/// Écrit un serveur sous la forme `hôte:port (chiffrement)`.
pub fn endpoint_label(endpoint: &ServerEndpoint) -> String {
    format!(
        "{}:{} ({})",
        endpoint.host,
        endpoint.port,
        security_label(endpoint.security)
    )
}

/// Écrit la liste des méthodes d'authentification, séparées par des virgules.
pub fn auth_list_label(methods: &[AuthMethod]) -> String {
    let labels: Vec<&str> = methods.iter().map(auth_label).collect();
    labels.join(", ")
}
