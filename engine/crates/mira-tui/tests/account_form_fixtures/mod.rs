//! Fournisseurs, serveurs et erreurs de test du formulaire d'ajout de compte.
#![allow(dead_code, unused_imports)]

mod flow;
mod replies;
mod screen;

use mira_assembly::mira_vocabulary::error::codes::authentication::REJECTED;
use mira_assembly::mira_vocabulary::{
    AuthMethod, ErrorCategory, MiraError, OAuthSettings, ProviderDescriptor, ProviderId,
    ProviderLimits, ProviderQuirks, Security, ServerEndpoint, ServerSettings,
};

pub use flow::{hit, ready_state, token_of, type_text};
pub use replies::{added_command, added_reply, prepared_reply};
pub use screen::{cursor_after_render, screen};

pub const SECRET: &str = "s3cret-VALUE";

/// Fiche de fournisseur de test.
pub fn provider(id: &str, domains: &[&str], methods: Vec<AuthMethod>) -> ProviderDescriptor {
    ProviderDescriptor {
        id: ProviderId::new(id),
        display_name: format!("Fournisseur {id}"),
        domains: domains.iter().map(|d| d.to_string()).collect(),
        auth_methods: methods,
        limits: ProviderLimits::default(),
        quirks: ProviderQuirks::default(),
    }
}

/// Fournisseur qui a ses domaines et propose mot de passe et mot de passe d'application.
pub fn known_provider() -> ProviderDescriptor {
    let methods = vec![AuthMethod::Password, AuthMethod::AppPassword];
    provider("acme", &["acme.test"], methods)
}

/// Fournisseur de repli : aucun domaine propre.
pub fn fallback_provider() -> ProviderDescriptor {
    provider("fallback", &[], vec![AuthMethod::Password])
}

/// Fournisseur qui ne propose qu'OAuth.
pub fn oauth_only_provider() -> ProviderDescriptor {
    let settings = OAuthSettings {
        authorize_url: "https://auth.example/authorize".to_string(),
        token_url: "https://auth.example/token".to_string(),
        scopes: Vec::new(),
    };
    provider("bigmail", &["big.test"], vec![AuthMethod::OAuth2(settings)])
}

/// Serveurs proposés par le moteur : IMAP 993 et SMTP 465, en TLS.
pub fn proposed_servers() -> ServerSettings {
    let endpoint = |host: &str, port: u16| ServerEndpoint {
        host: host.to_string(),
        port,
        security: Security::Tls,
    };
    ServerSettings {
        imap: endpoint("imap.example.org", 993),
        smtp: endpoint("smtp.example.org", 465),
    }
}

/// Erreur de connexion telle que le moteur la rend : un message sans secret.
pub fn login_error() -> MiraError {
    MiraError::new(
        ErrorCategory::Authentication,
        REJECTED,
        "identifiants refusés par le serveur",
    )
}
