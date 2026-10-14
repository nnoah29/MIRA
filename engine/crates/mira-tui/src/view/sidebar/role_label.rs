//! Libellé français d'une boîte selon son rôle.

use mira_assembly::mira_vocabulary::{Mailbox, MailboxRole};

/// Nom affiché d'une boîte : libellé du rôle, ou nom d'affichage pour les autres.
pub fn label(mailbox: &Mailbox) -> String {
    match mailbox.role {
        MailboxRole::Inbox => "Réception".to_string(),
        MailboxRole::Sent => "Envoyés".to_string(),
        MailboxRole::Drafts => "Brouillons".to_string(),
        MailboxRole::Trash => "Corbeille".to_string(),
        MailboxRole::Junk => "Indésirables".to_string(),
        MailboxRole::Archive => "Archives".to_string(),
        MailboxRole::All => "Tous les messages".to_string(),
        MailboxRole::Flagged => "Suivis".to_string(),
        MailboxRole::Other => mailbox.display_name.clone(),
    }
}
