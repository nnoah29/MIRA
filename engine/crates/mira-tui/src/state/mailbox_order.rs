//! Ordre d'affichage des boîtes : rôles système d'abord, puis nom.

use mira_assembly::mira_vocabulary::{Mailbox, MailboxRole};

/// Rang d'affichage d'un rôle de boîte.
fn role_rank(role: MailboxRole) -> u8 {
    match role {
        MailboxRole::Inbox => 0,
        MailboxRole::Drafts => 1,
        MailboxRole::Sent => 2,
        MailboxRole::Archive => 3,
        MailboxRole::Junk => 4,
        MailboxRole::Trash => 5,
        MailboxRole::All => 6,
        MailboxRole::Flagged => 7,
        MailboxRole::Other => 8,
    }
}

/// Trie les boîtes par rôle puis par nom d'affichage.
pub fn sort_mailboxes(mailboxes: &mut [Mailbox]) {
    mailboxes.sort_by(|left, right| {
        role_rank(left.role)
            .cmp(&role_rank(right.role))
            .then_with(|| left.display_name.cmp(&right.display_name))
    });
}
