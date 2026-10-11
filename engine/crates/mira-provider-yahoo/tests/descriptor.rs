#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Serveurs, authentification, limites et particularités du service (6.7).

use mira_provider_yahoo::YahooProvider;
use mira_vocabulary::{
    ArchiveStrategy, AuthMethod, EmailAddress, MailboxModel, Provider, ProviderLimits, Security,
};

/// Les serveurs sont fixes et chiffrés par TLS implicite
#[test]
fn servers_are_fixed_and_use_implicit_tls() {
    let address = EmailAddress::parse("moi@rocketmail.com").unwrap();
    let servers = YahooProvider.servers_for(&address);
    assert_eq!(servers.imap.host, "imap.mail.yahoo.com");
    assert_eq!(servers.imap.port, 993);
    assert_eq!(servers.imap.security, Security::Tls);
    assert_eq!(servers.smtp.host, "smtp.mail.yahoo.com");
    assert_eq!(servers.smtp.port, 465);
    assert_eq!(servers.smtp.security, Security::Tls);
}

/// Le mot de passe d'application est la seule méthode
#[test]
fn app_password_is_the_only_method() {
    let descriptor = YahooProvider.descriptor();
    assert_eq!(descriptor.auth_methods, vec![AuthMethod::AppPassword]);
}

/// Limites de politesse et particularités déclarées
#[test]
fn limits_and_quirks_match_the_data_table() {
    let descriptor = YahooProvider.descriptor();
    assert_eq!(descriptor.id.as_str(), "yahoo");
    assert_eq!(
        descriptor.limits,
        ProviderLimits {
            max_connections: 3,
            min_poll_interval_secs: 300,
        }
    );
    assert_eq!(descriptor.quirks.mailbox_model, MailboxModel::Folders);
    assert_eq!(
        descriptor.quirks.archive,
        ArchiveStrategy::MoveToArchiveMailbox
    );
    assert!(!descriptor.quirks.server_saves_sent_copy);
    assert!(descriptor.quirks.unreliable_extensions.is_empty());
}
