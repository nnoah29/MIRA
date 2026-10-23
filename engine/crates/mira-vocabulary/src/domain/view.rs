//! Vues de liste : une boîte, un libellé local ou une vue standard.

use serde::{Deserialize, Serialize};

use crate::ids::kinds::{LocalLabelId, MailboxId};

/// Vue standard construite sur tous les comptes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StandardView {
    Unified,
    Unread,
    Flagged,
}

/// Référence à ce que l'on veut lister.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ViewRef {
    Mailbox { mailbox: MailboxId },
    LocalLabel { label: LocalLabelId },
    Standard { view: StandardView },
}
