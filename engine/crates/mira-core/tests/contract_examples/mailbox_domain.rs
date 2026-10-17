//! Exemples du domaine des boîtes (lecture et sélection des boîtes synchronisées).

use mira_core::api::MailboxRequest;

pub const NAMES: &[&str] = &["mailbox.list_mailboxes", "mailbox.set_synced_mailboxes"];

pub const EXAMPLES: &[&str] = &[
    r#"{"domain":"mailbox","request":{"op":"list_mailboxes","account":"a"}}"#,
    r#"{"domain":"mailbox","request":{"op":"set_synced_mailboxes","account":"a","mailboxes":["m1","m2"]}}"#,
];

/// Nom `domaine.opération` d'une requête du domaine boîtes
pub fn mailbox_name(request: &MailboxRequest) -> &'static str {
    match request {
        MailboxRequest::ListMailboxes { .. } => "mailbox.list_mailboxes",
        MailboxRequest::SetSyncedMailboxes { .. } => "mailbox.set_synced_mailboxes",
    }
}
