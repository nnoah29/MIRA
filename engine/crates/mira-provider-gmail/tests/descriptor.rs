#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Serveurs, authentifications, limites et particularités du service (6.7).

use mira_provider_gmail::GmailProvider;
use mira_vocabulary::{
    ArchiveStrategy, AuthMethod, EmailAddress, MailboxModel, Provider, ProviderLimits, Security,
};

/// Les serveurs sont fixes et chiffrés par TLS implicite
#[test]
fn servers_are_fixed_and_use_implicit_tls() {
    let address = EmailAddress::parse("moi@googlemail.com").unwrap();
    let servers = GmailProvider.servers_for(&address);
    assert_eq!(servers.imap.host, "imap.gmail.com");
    assert_eq!(servers.imap.port, 993);
    assert_eq!(servers.imap.security, Security::Tls);
    assert_eq!(servers.smtp.host, "smtp.gmail.com");
    assert_eq!(servers.smtp.port, 465);
    assert_eq!(servers.smtp.security, Security::Tls);
}

/// OAuth2 vient d'abord, avec ses adresses et sa portée, puis le mot de passe d'application
#[test]
fn oauth_comes_first_then_app_password() {
    let descriptor = GmailProvider.descriptor();
    assert_eq!(descriptor.auth_methods.len(), 2);
    match &descriptor.auth_methods[0] {
        AuthMethod::OAuth2(settings) => {
            assert_eq!(
                settings.authorize_url,
                "https://accounts.google.com/o/oauth2/v2/auth"
            );
            assert_eq!(settings.token_url, "https://oauth2.googleapis.com/token");
            assert_eq!(
                settings.scopes,
                vec!["https://mail.google.com/".to_string()]
            );
        }
        other => panic!("OAuth2 attendu en premier, obtenu {other:?}"),
    }
    assert_eq!(descriptor.auth_methods[1], AuthMethod::AppPassword);
}

/// Limites de politesse et particularités déclarées
#[test]
fn limits_and_quirks_match_the_data_table() {
    let descriptor = GmailProvider.descriptor();
    assert_eq!(descriptor.id.as_str(), "gmail");
    assert_eq!(
        descriptor.limits,
        ProviderLimits {
            max_connections: 5,
            min_poll_interval_secs: 120,
        }
    );
    assert_eq!(descriptor.domains, vec!["gmail.com", "googlemail.com"]);
    assert_eq!(descriptor.quirks.mailbox_model, MailboxModel::Labels);
    assert_eq!(descriptor.quirks.archive, ArchiveStrategy::RemoveFromInbox);
    assert!(descriptor.quirks.server_saves_sent_copy);
    assert_eq!(
        descriptor.quirks.extra_fetch_attributes,
        vec!["X-GM-MSGID", "X-GM-THRID", "X-GM-LABELS"]
    );
}
