#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Rôles des boîtes propres au service, avec leurs noms réels.

use mira_provider_yahoo::YahooProvider;
use mira_vocabulary::{MailboxRole, Provider, RawMailbox};

/// Demande au service l'indice de rôle d'une boîte portant ce nom
fn hint(name: &str) -> Option<MailboxRole> {
    YahooProvider.mailbox_role_hint(&RawMailbox {
        name: name.to_string(),
        delimiter: Some("/".to_string()),
        attributes: Vec::new(),
    })
}

/// Table des noms réels vers leurs rôles
#[test]
fn real_names_map_to_their_roles() {
    let table = [
        ("Sent", MailboxRole::Sent),
        ("Draft", MailboxRole::Drafts),
        ("Trash", MailboxRole::Trash),
        ("Bulk Mail", MailboxRole::Junk),
        ("Bulk", MailboxRole::Junk),
        ("Archive", MailboxRole::Archive),
        ("bulk mail", MailboxRole::Junk),
    ];
    for (name, role) in table {
        assert_eq!(hint(name), Some(role), "{name}");
    }
}

/// Les noms génériques ou étrangers au service ne reçoivent aucun indice
#[test]
fn other_names_get_no_hint() {
    for name in ["INBOX", "Sent Items", "Spam", "[Gmail]/Trash", "Travail"] {
        assert_eq!(hint(name), None, "{name}");
    }
}
