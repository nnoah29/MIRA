//! Faux provider OAuth2 des tests du CLI : gère le domaine `oauth.test`.

use mira_assembly::mira_vocabulary::{
    AuthMethod, EmailAddress, MailboxRole, OAuthSettings, Provider, ProviderDescriptor, ProviderId,
    ProviderLimits, ProviderQuirks, RawMailbox, RawMessageAttributes, Security, ServerEndpoint,
    ServerMessageTranslation, ServerSettings,
};

pub const OAUTH_PROVIDER_ID: &str = "fake_oauth";

/// Faux provider qui propose OAuth2 ; ses adresses d'autorisation sont des valeurs fictives.
#[derive(Debug, Clone, Copy, Default)]
pub struct FakeOAuthProvider;

impl Provider for FakeOAuthProvider {
    /// Décrit un service OAuth2 sur `oauth.test`.
    fn descriptor(&self) -> ProviderDescriptor {
        ProviderDescriptor {
            id: ProviderId::new(OAUTH_PROVIDER_ID),
            display_name: "Fake OAuth".to_string(),
            domains: vec!["oauth.test".to_string()],
            auth_methods: vec![AuthMethod::OAuth2(OAuthSettings {
                authorize_url: "https://auth.oauth.test/authorize".to_string(),
                token_url: "https://auth.oauth.test/token".to_string(),
                scopes: vec!["mail".to_string()],
            })],
            limits: ProviderLimits::default(),
            quirks: ProviderQuirks::default(),
        }
    }

    /// Gère exactement le domaine `oauth.test`.
    fn handles_address(&self, address: &EmailAddress) -> bool {
        address.domain() == "oauth.test"
    }

    /// Donne des serveurs TLS fictifs.
    fn servers_for(&self, address: &EmailAddress) -> ServerSettings {
        let endpoint = |prefix: &str, port: u16| ServerEndpoint {
            host: format!("{prefix}.{}", address.domain()),
            port,
            security: Security::Tls,
        };
        ServerSettings {
            imap: endpoint("imap", 993),
            smtp: endpoint("smtp", 465),
        }
    }

    /// Ne propose aucun rôle particulier.
    fn mailbox_role_hint(&self, _mailbox: &RawMailbox) -> Option<MailboxRole> {
        None
    }

    /// Ne traduit rien.
    fn translate_attributes(&self, _raw: &RawMessageAttributes) -> ServerMessageTranslation {
        ServerMessageTranslation::default()
    }
}
