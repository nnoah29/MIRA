//! Détection des actions contradictoires dans une même règle.

use mira_vocabulary::{MiraError, Rule, RuleAction};

use crate::actions::errors::input::invalid_parameter;

/// Vrai si les deux actions ne peuvent pas s'appliquer ensemble.
fn contradicts(first: &RuleAction, second: &RuleAction) -> bool {
    use RuleAction::{Archive, MarkFlagged, MarkRead, MarkSpam, MoveTo, Trash};
    match (first, second) {
        (Trash, Archive) | (Archive, Trash) => true,
        (MoveTo { .. }, Trash | Archive) | (Trash | Archive, MoveTo { .. }) => true,
        (MoveTo { mailbox: a }, MoveTo { mailbox: b }) => a != b,
        (MarkRead { read: a }, MarkRead { read: b }) => a != b,
        (MarkFlagged { flagged: a }, MarkFlagged { flagged: b }) => a != b,
        (MarkSpam { spam: a }, MarkSpam { spam: b }) => a != b,
        _ => false,
    }
}

/// Refuse une règle dont deux actions se contredisent.
pub(super) fn check_conflicts(rule: &Rule) -> Result<(), MiraError> {
    for (index, first) in rule.actions.iter().enumerate() {
        let later = &rule.actions[index + 1..];
        if later.iter().any(|second| contradicts(first, second)) {
            return Err(invalid_parameter(
                "la règle contient des actions contradictoires",
            ));
        }
    }
    Ok(())
}
