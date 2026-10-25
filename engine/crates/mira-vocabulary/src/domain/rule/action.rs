//! Catalogue des actions qu'une règle peut déclencher.

use serde::{Deserialize, Serialize};

use crate::ids::kinds::{LocalLabelId, MailboxId, TemplateId};

/// Action déclenchée par une règle ; la plupart se traduisent en `ActionKind`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RuleAction {
    MarkRead { read: bool },
    MarkFlagged { flagged: bool },
    MoveTo { mailbox: MailboxId },
    AddToMailbox { mailbox: MailboxId },
    RemoveFromMailbox { mailbox: MailboxId },
    Archive,
    Trash,
    MarkSpam { spam: bool },
    AddLocalLabel { label: LocalLabelId },
    RemoveLocalLabel { label: LocalLabelId },
    SuppressNotification,
    StopProcessing,
    ReplyWithTemplate { template: TemplateId },
}
