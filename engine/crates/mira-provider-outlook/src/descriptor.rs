//! Fiche du provider outlook : domaines, OAuth2 seulement, limites, dossiers (6.7, à vérifier).

use mira_vocabulary::{
    AuthMethod, OAuthSettings, ProviderDescriptor, ProviderId, ProviderLimits, ProviderQuirks,
};

/// Identifiant du provider dans le catalogue.
pub const PROVIDER_ID: &str = "outlook";
/// Domaines gérés, en minuscules et sans sous-domaine, variantes régionales comprises.
pub const DOMAINS: &[&str] = &[
    "outlook.com",
    "hotmail.com",
    "live.com",
    "msn.com",
    "hotmail.fr",
    "outlook.fr",
    "live.fr",
    "hotmail.co.uk",
    "outlook.de",
    "hotmail.de",
    "hotmail.it",
    "outlook.es",
    "hotmail.es",
    "live.co.uk",
    "live.de",
    "live.it",
    "outlook.be",
    "outlook.jp",
    "hotmail.be",
    "live.ca",
    "hotmail.ca",
    "live.com.au",
    "outlook.com.au",
];
/// Adresse d'autorisation OAuth2 (comptes personnels).
pub const AUTHORIZE_URL: &str = "https://login.microsoftonline.com/consumers/oauth2/v2.0/authorize";
/// Adresse d'échange de jetons OAuth2 (comptes personnels).
pub const TOKEN_URL: &str = "https://login.microsoftonline.com/consumers/oauth2/v2.0/token";
/// Portées OAuth2 demandées pour IMAP, SMTP et le renouvellement.
pub const SCOPES: &[&str] = &[
    "https://outlook.office.com/IMAP.AccessAsUser.All",
    "https://outlook.office.com/SMTP.Send",
    "offline_access",
];

/// Construit la méthode OAuth2 du service, la seule proposée.
fn oauth_method() -> AuthMethod {
    AuthMethod::OAuth2(OAuthSettings {
        authorize_url: AUTHORIZE_URL.to_string(),
        token_url: TOKEN_URL.to_string(),
        scopes: SCOPES.iter().map(ToString::to_string).collect(),
    })
}

/// Construit la fiche statique du provider outlook (dossiers, copie des envoyés par le serveur).
pub fn descriptor() -> ProviderDescriptor {
    ProviderDescriptor {
        id: ProviderId::new(PROVIDER_ID),
        display_name: "Outlook".to_string(),
        domains: DOMAINS.iter().map(ToString::to_string).collect(),
        auth_methods: vec![oauth_method()],
        limits: ProviderLimits {
            max_connections: 4,
            min_poll_interval_secs: 120,
        },
        quirks: ProviderQuirks {
            server_saves_sent_copy: true,
            ..ProviderQuirks::default()
        },
    }
}
