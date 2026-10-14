//! Requêtes du domaine des fils de conversation (12.4).

use mira_vocabulary::{MailboxId, PageRequest, ThreadId};
use serde::{Deserialize, Serialize};

/// Lecture des fils d'une boîte et d'un fil.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum ThreadRequest {
    ListThreads {
        mailbox: MailboxId,
        page: PageRequest,
    },
    GetThread {
        thread: ThreadId,
    },
}
