//! Traduction des actions de règle en actions de file.

use super::outcome::RuleOutcome;
use mira_vocabulary::{ActionKind, RuleAction};

/// Ajoute au résultat les actions d'une règle ; vrai si elle demande l'arrêt du parcours.
pub fn apply_actions(actions: &[RuleAction], outcome: &mut RuleOutcome) -> bool {
    let mut stop = false;
    for action in actions {
        match translate(action) {
            Some(kind) => outcome.actions.push(kind),
            None if *action == RuleAction::StopProcessing => stop = true,
            None => outcome.silenced = true,
        }
    }
    stop
}

/// Action de file équivalente, ou rien pour le silence et l'arrêt.
fn translate(action: &RuleAction) -> Option<ActionKind> {
    match action {
        RuleAction::MarkRead { read } => Some(ActionKind::MarkRead { read: *read }),
        RuleAction::MarkFlagged { flagged } => Some(ActionKind::MarkFlagged { flagged: *flagged }),
        RuleAction::MoveTo { mailbox } => Some(ActionKind::Move {
            to: mailbox.clone(),
        }),
        RuleAction::AddToMailbox { mailbox } => Some(ActionKind::AddToMailbox {
            mailbox: mailbox.clone(),
        }),
        RuleAction::RemoveFromMailbox { mailbox } => Some(ActionKind::RemoveFromMailbox {
            mailbox: mailbox.clone(),
        }),
        RuleAction::Archive => Some(ActionKind::Archive),
        RuleAction::Trash => Some(ActionKind::Trash),
        RuleAction::MarkSpam { spam } => Some(ActionKind::MarkSpam { spam: *spam }),
        RuleAction::SuppressNotification | RuleAction::StopProcessing => None,
    }
}
