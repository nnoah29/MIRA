//! Exemples couvrant chaque variante de condition et d'action de règle.

mod numeric_conditions;
mod state_conditions;
mod text_conditions;

use mira_vocabulary::{ConditionTest, MailboxId, RuleAction};

/// Liste une instance de chaque variante de condition.
pub fn every_condition_test() -> Vec<ConditionTest> {
    let mut all = text_conditions::text_conditions();
    all.extend(numeric_conditions::numeric_conditions());
    all.extend(state_conditions::state_conditions());
    all
}

/// Liste une instance de chaque variante d'action de règle.
pub fn every_rule_action() -> Vec<RuleAction> {
    vec![
        RuleAction::MarkRead { read: true },
        RuleAction::MarkFlagged { flagged: false },
        RuleAction::MoveTo {
            mailbox: MailboxId::from_raw("mb-2"),
        },
        RuleAction::AddToMailbox {
            mailbox: MailboxId::from_raw("mb-3"),
        },
        RuleAction::RemoveFromMailbox {
            mailbox: MailboxId::from_raw("mb-4"),
        },
        RuleAction::Archive,
        RuleAction::Trash,
        RuleAction::MarkSpam { spam: true },
        RuleAction::SuppressNotification,
        RuleAction::StopProcessing,
    ]
}
