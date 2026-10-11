#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Serveurs, authentification, limites et particularités du service (6.7).

use mira_provider_outlook::OutlookProvider;
use mira_vocabulary::{
    ArchiveStrategy, AuthMethod, EmailAddress, MailboxModel, Provider, ProviderLimits, Security,
};

/// Le serveur IMAP est en TLS implicite et le SMTP en STARTTLS
#[test]
fn servers_use_tls_for_imap_and_starttls_for_smtp() {
    let address = EmailAddress::parse("moi@hotmail.fr").unwrap();
    let servers = OutlookProvider.servers_for(&address);
    assert_eq!(servers.imap.host, "outlook.office365.com");
    assert_eq!(servers.imap.port, 993);
    assert_eq!(servers.imap.security, Security::Tls);
    assert_eq!(servers.smtp.host, "smtp.office365.com");
    assert_eq!(servers.smtp.port, 587);
    assert_eq!(servers.smtp.security, Security::StartTls);
}

/// OAuth2 est la seule méthode, avec ses trois portées
#[test]
fn oauth_is_the_only_method() {
    let descriptor = OutlookProvider.descriptor();
    assert_eq!(descriptor.auth_methods.len(), 1);
    match &descriptor.auth_methods[0] {
        AuthMethod::OAuth2(settings) => {
            assert!(
                settings
                    .authorize_url
                    .ends_with("/consumers/oauth2/v2.0/authorize")
            );
            assert!(settings.token_url.ends_with("/consumers/oauth2/v2.0/token"));
            assert_eq!(
                settings.scopes,
                vec![
                    "https://outlook.office.com/IMAP.AccessAsUser.All",
                    "https://outlook.office.com/SMTP.Send",
                    "offline_access",
                ]
            );
        }
        other => panic!("OAuth2 attendu, obtenu {other:?}"),
    }
}

/// Limites de politesse et particularités déclarées
#[test]
fn limits_and_quirks_match_the_data_table() {
    let descriptor = OutlookProvider.descriptor();
    assert_eq!(descriptor.id.as_str(), "outlook");
    assert_eq!(
        descriptor.limits,
        ProviderLimits {
            max_connections: 4,
            min_poll_interval_secs: 120,
        }
    );
    assert_eq!(descriptor.quirks.mailbox_model, MailboxModel::Folders);
    assert_eq!(
        descriptor.quirks.archive,
        ArchiveStrategy::MoveToArchiveMailbox
    );
    assert!(descriptor.quirks.server_saves_sent_copy);
    assert!(descriptor.quirks.extra_fetch_attributes.is_empty());
    assert!(descriptor.domains.contains(&"outlook.com".to_string()));
}
