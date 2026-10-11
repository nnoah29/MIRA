#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Rôles des boîtes propres au service, avec leurs noms réels.

use mira_provider_outlook::OutlookProvider;
use mira_vocabulary::{MailboxRole, Provider, RawMailbox};

/// Demande au service l'indice de rôle d'une boîte portant ce nom
fn hint(name: &str) -> Option<MailboxRole> {
    OutlookProvider.mailbox_role_hint(&RawMailbox {
        name: name.to_string(),
        delimiter: Some("/".to_string()),
        attributes: Vec::new(),
    })
}

/// Table des noms réels vers leurs rôles
#[test]
fn real_names_map_to_their_roles() {
    let table = [
        ("Sent Items", MailboxRole::Sent),
        ("Deleted Items", MailboxRole::Trash),
        ("Junk Email", MailboxRole::Junk),
        ("Junk", MailboxRole::Junk),
        ("Archive", MailboxRole::Archive),
        ("Drafts", MailboxRole::Drafts),
        ("sent items", MailboxRole::Sent),
    ];
    for (name, role) in table {
        assert_eq!(hint(name), Some(role), "{name}");
    }
}

/// Les noms génériques ou étrangers au service ne reçoivent aucun indice
#[test]
fn other_names_get_no_hint() {
    for name in ["INBOX", "Sent", "Trash", "[Gmail]/Sent Mail", "Travail"] {
        assert_eq!(hint(name), None, "{name}");
    }
}
