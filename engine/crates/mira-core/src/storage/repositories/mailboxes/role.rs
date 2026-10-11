//! Nom stocké du rôle d'une boîte.

use mira_vocabulary::error::codes::local_storage;
use mira_vocabulary::{ErrorCategory, MailboxRole, MiraError};

const ALL_ROLES: [MailboxRole; 9] = [
    MailboxRole::Inbox,
    MailboxRole::Sent,
    MailboxRole::Drafts,
    MailboxRole::Trash,
    MailboxRole::Junk,
    MailboxRole::Archive,
    MailboxRole::All,
    MailboxRole::Flagged,
    MailboxRole::Other,
];

/// Donne le nom stocké d'un rôle de boîte
pub(super) fn role_name(role: MailboxRole) -> &'static str {
    match role {
        MailboxRole::Inbox => "inbox",
        MailboxRole::Sent => "sent",
        MailboxRole::Drafts => "drafts",
        MailboxRole::Trash => "trash",
        MailboxRole::Junk => "junk",
        MailboxRole::Archive => "archive",
        MailboxRole::All => "all",
        MailboxRole::Flagged => "flagged",
        MailboxRole::Other => "other",
    }
}

/// Relit un rôle de boîte depuis son nom stocké
pub(super) fn role_from_name(name: &str) -> Result<MailboxRole, MiraError> {
    ALL_ROLES
        .into_iter()
        .find(|role| role_name(*role) == name)
        .ok_or_else(|| {
            MiraError::new(
                ErrorCategory::LocalStorage,
                local_storage::CORRUPTED,
                "rôle de boîte inconnu en base",
            )
        })
}
