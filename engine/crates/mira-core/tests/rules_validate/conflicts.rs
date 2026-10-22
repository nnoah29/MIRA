//! Actions contradictoires dans une même règle.

use crate::world::{valid_rule, world};
use mira_core::rules::validate::validate_rule;
use mira_vocabulary::{MailboxId, RuleAction};

/// Vrai si la règle valide munie de ces actions est refusée
fn refused(actions: Vec<RuleAction>) -> bool {
    let mut rule = valid_rule();
    rule.actions = actions;
    validate_rule(&rule, &world()).is_err()
}

/// Corbeille et archive se contredisent
#[test]
fn trash_and_archive_conflict() {
    assert!(refused(vec![RuleAction::Trash, RuleAction::Archive]));
    assert!(refused(vec![RuleAction::Archive, RuleAction::Trash]));
}

/// Un déplacement contredit la corbeille et l'archive
#[test]
fn move_conflicts_with_trash_and_archive() {
    let moved = RuleAction::MoveTo {
        mailbox: MailboxId::from_raw("mb-1"),
    };
    assert!(refused(vec![moved.clone(), RuleAction::Trash]));
    assert!(refused(vec![RuleAction::Archive, moved]));
}

/// Des valeurs opposées du même drapeau se contredisent
#[test]
fn opposite_flag_values_conflict() {
    assert!(refused(vec![
        RuleAction::MarkRead { read: true },
        RuleAction::MarkRead { read: false },
    ]));
    assert!(refused(vec![
        RuleAction::MarkFlagged { flagged: false },
        RuleAction::MarkFlagged { flagged: true },
    ]));
    assert!(refused(vec![
        RuleAction::MarkSpam { spam: true },
        RuleAction::MarkSpam { spam: false },
    ]));
}

/// Des actions compatibles sont acceptées
#[test]
fn compatible_actions_are_accepted() {
    assert!(!refused(vec![
        RuleAction::MarkRead { read: true },
        RuleAction::MarkFlagged { flagged: true },
        RuleAction::SuppressNotification,
        RuleAction::StopProcessing,
    ]));
}
