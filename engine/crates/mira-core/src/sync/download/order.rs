//! Ordre de synchronisation des boîtes (10.12) : la réception d'abord, puis les boîtes utiles, puis le reste.

use mira_vocabulary::{Mailbox, MailboxRole};

/// Priorité d'un rôle : plus petit = plus tôt.
fn priority(role: MailboxRole) -> u8 {
    match role {
        MailboxRole::Inbox => 0,
        MailboxRole::Sent => 1,
        MailboxRole::Drafts => 2,
        MailboxRole::Archive | MailboxRole::All | MailboxRole::Flagged => 3,
        MailboxRole::Other => 4,
        MailboxRole::Trash => 5,
        MailboxRole::Junk => 6,
    }
}

/// Boîtes à synchroniser, la réception en premier, puis par priorité de rôle et par nom.
pub fn sync_order(mut mailboxes: Vec<Mailbox>) -> Vec<Mailbox> {
    mailboxes.retain(|mailbox| mailbox.sync_enabled);
    mailboxes.sort_by(|a, b| (priority(a.role), &a.name).cmp(&(priority(b.role), &b.name)));
    mailboxes
}
