//! Photographie de l'activité de synchronisation d'un compte.

use mira_vocabulary::{MailboxId, Timestamp};

/// Ce que le superviseur sait de l'activité d'un compte à un instant donné.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SyncReport {
    pub running: bool,
    pub last_success: Option<Timestamp>,
    pub last_error: Option<String>,
    pub mailboxes: Vec<MailboxId>,
}
