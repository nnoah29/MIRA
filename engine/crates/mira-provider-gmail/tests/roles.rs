#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Rôles des boîtes propres au service, sous les deux préfixes.

use mira_provider_gmail::GmailProvider;
use mira_vocabulary::{MailboxRole, Provider, RawMailbox};

/// Demande au service l'indice de rôle d'une boîte portant ce nom
fn hint(name: &str) -> Option<MailboxRole> {
    GmailProvider.mailbox_role_hint(&RawMailbox {
        name: name.to_string(),
        delimiter: Some("/".to_string()),
        attributes: Vec::new(),
    })
}

/// Table des noms réels vers leurs rôles, pour les deux préfixes
#[test]
fn real_names_map_to_their_roles() {
    let table = [
        ("Sent Mail", MailboxRole::Sent),
        ("Drafts", MailboxRole::Drafts),
        ("Trash", MailboxRole::Trash),
        ("Spam", MailboxRole::Junk),
        ("All Mail", MailboxRole::All),
        ("Starred", MailboxRole::Flagged),
    ];
    for (suffix, role) in table {
        for prefix in ["[Gmail]", "[Google Mail]"] {
            assert_eq!(
                hint(&format!("{prefix}/{suffix}")),
                Some(role),
                "{prefix}/{suffix}"
            );
        }
    }
}

/// La casse ne compte pas
#[test]
fn matching_ignores_case() {
    assert_eq!(hint("[GMAIL]/sent mail"), Some(MailboxRole::Sent));
    assert_eq!(hint("[google mail]/ALL MAIL"), Some(MailboxRole::All));
}

/// Les noms étrangers au service ne reçoivent aucun indice
#[test]
fn foreign_names_get_no_hint() {
    for name in [
        "INBOX",
        "Sent Mail",
        "Sent Items",
        "[Gmail]",
        "[Gmail]/Important",
        "Travail",
    ] {
        assert_eq!(hint(name), None, "{name}");
    }
}
