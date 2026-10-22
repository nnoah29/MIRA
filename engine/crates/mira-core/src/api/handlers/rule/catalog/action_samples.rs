//! Une instance de chaque variante d'action, dont le catalogue lit l'étiquette et les paramètres.

use mira_vocabulary::{LocalLabelId, MailboxId, RuleAction};

/// Une instance de chaque variante d'action, dans l'ordre du catalogue
pub(super) fn action_samples() -> Vec<RuleAction> {
    let mailbox = MailboxId::from_raw("");
    vec![
        RuleAction::MarkRead { read: true },
        RuleAction::MarkFlagged { flagged: true },
        RuleAction::MoveTo {
            mailbox: mailbox.clone(),
        },
        RuleAction::AddToMailbox {
            mailbox: mailbox.clone(),
        },
        RuleAction::RemoveFromMailbox { mailbox },
        RuleAction::Archive,
        RuleAction::Trash,
        RuleAction::MarkSpam { spam: true },
        RuleAction::AddLocalLabel {
            label: LocalLabelId::from_raw(""),
        },
        RuleAction::RemoveLocalLabel {
            label: LocalLabelId::from_raw(""),
        },
        RuleAction::SuppressNotification,
        RuleAction::StopProcessing,
    ]
}
