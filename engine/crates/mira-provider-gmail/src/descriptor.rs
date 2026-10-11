//! Fiche du provider gmail : domaines, OAuth2 puis mot de passe d'application, limites, particularités (6.7, à vérifier).

use mira_vocabulary::{
    ArchiveStrategy, AuthMethod, MailboxModel, OAuthSettings, ProviderDescriptor, ProviderId,
    ProviderLimits, ProviderQuirks,
};

/// Identifiant du provider dans le catalogue.
pub const PROVIDER_ID: &str = "gmail";
/// Domaines gérés, en minuscules et sans sous-domaine.
pub const DOMAINS: &[&str] = &["gmail.com", "googlemail.com"];
/// Adresse d'autorisation OAuth2.
pub const AUTHORIZE_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
/// Adresse d'échange de jetons OAuth2.
pub const TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
/// Portée OAuth2 demandée pour IMAP et SMTP.
pub const SCOPE: &str = "https://mail.google.com/";
/// Attributs FETCH propres au service.
pub const EXTRA_FETCH_ATTRIBUTES: &[&str] = &["X-GM-MSGID", "X-GM-THRID", "X-GM-LABELS"];

/// Construit la méthode OAuth2 du service.
fn oauth_method() -> AuthMethod {
    AuthMethod::OAuth2(OAuthSettings {
        authorize_url: AUTHORIZE_URL.to_string(),
        token_url: TOKEN_URL.to_string(),
        scopes: vec![SCOPE.to_string()],
    })
}

/// Construit les particularités : libellés, archivage par retrait de la boîte de réception.
fn quirks() -> ProviderQuirks {
    ProviderQuirks {
        mailbox_model: MailboxModel::Labels,
        archive: ArchiveStrategy::RemoveFromInbox,
        server_saves_sent_copy: true,
        unreliable_extensions: Vec::new(),
        extra_fetch_attributes: EXTRA_FETCH_ATTRIBUTES
            .iter()
            .map(ToString::to_string)
            .collect(),
    }
}

/// Construit la fiche statique du provider gmail.
pub fn descriptor() -> ProviderDescriptor {
    ProviderDescriptor {
        id: ProviderId::new(PROVIDER_ID),
        display_name: "Gmail".to_string(),
        domains: DOMAINS.iter().map(ToString::to_string).collect(),
        auth_methods: vec![oauth_method(), AuthMethod::AppPassword],
        limits: ProviderLimits {
            max_connections: 5,
            min_poll_interval_secs: 120,
        },
        quirks: quirks(),
    }
}
