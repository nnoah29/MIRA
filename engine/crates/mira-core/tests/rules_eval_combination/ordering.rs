//! Ordre des règles : concaténation des actions, dernier mot, arrêt du parcours.

use crate::support_rules::fixtures::base_context;
use crate::support_rules::probe::{cond, rule};
use mira_core::rules::evaluate::evaluate;
use mira_vocabulary::{ActionKind, ConditionTest, MailboxId, Rule, RuleAction, RuleId};

/// Règle toujours vraie portant ces actions
fn always(id: &str, actions: Vec<RuleAction>) -> Rule {
    rule(id, vec![vec![cond(ConditionTest::HasAttachment)]], actions)
}

/// Les actions de drapeau et d'archivage se traduisent directement
#[test]
fn flag_and_archive_actions_translate_directly() {
    let actions = vec![
        RuleAction::MarkRead { read: true },
        RuleAction::MarkFlagged { flagged: true },
        RuleAction::Archive,
        RuleAction::Trash,
        RuleAction::MarkSpam { spam: true },
    ];
    let outcome = evaluate(&[always("r", actions)], &base_context());
    assert_eq!(
        outcome.actions,
        vec![
            ActionKind::MarkRead { read: true },
            ActionKind::MarkFlagged { flagged: true },
            ActionKind::Archive,
            ActionKind::Trash,
            ActionKind::MarkSpam { spam: true },
        ]
    );
    assert!(!outcome.silenced);
}

/// Les actions de boîte se traduisent, `MoveTo` devient `Move`
#[test]
fn mailbox_actions_translate_to_queue_actions() {
    let mailbox = MailboxId::from_raw("box-x");
    let to_box = |mailbox: &MailboxId| mailbox.clone();
    let actions = vec![
        RuleAction::MoveTo {
            mailbox: to_box(&mailbox),
        },
        RuleAction::AddToMailbox {
            mailbox: to_box(&mailbox),
        },
        RuleAction::RemoveFromMailbox {
            mailbox: to_box(&mailbox),
        },
    ];
    let outcome = evaluate(&[always("r", actions)], &base_context());
    assert_eq!(
        outcome.actions,
        vec![
            ActionKind::Move {
                to: to_box(&mailbox)
            },
            ActionKind::AddToMailbox {
                mailbox: to_box(&mailbox)
            },
            ActionKind::RemoveFromMailbox { mailbox },
        ]
    );
}

/// `SuppressNotification` n'ajoute pas d'action mais réduit au silence
#[test]
fn suppress_notification_sets_silenced() {
    let rules = [always("r", vec![RuleAction::SuppressNotification])];
    let outcome = evaluate(&rules, &base_context());
    assert!(outcome.silenced);
    assert!(outcome.actions.is_empty());
    assert_eq!(outcome.matched, vec![RuleId::from_raw("r")]);
}

/// Les actions s'ajoutent dans l'ordre des règles : la dernière gagne sur un même drapeau
#[test]
fn later_rule_wins_on_same_flag() {
    let rules = [
        always("a", vec![RuleAction::MarkRead { read: true }]),
        always("b", vec![RuleAction::MarkRead { read: false }]),
    ];
    let outcome = evaluate(&rules, &base_context());
    assert_eq!(
        outcome.actions,
        vec![
            ActionKind::MarkRead { read: true },
            ActionKind::MarkRead { read: false },
        ]
    );
}

/// `StopProcessing` garde les actions de sa règle et ignore les suivantes
#[test]
fn stop_processing_halts_later_rules() {
    let rules = [
        always("a", vec![RuleAction::Archive, RuleAction::StopProcessing]),
        always("b", vec![RuleAction::Trash]),
    ];
    let outcome = evaluate(&rules, &base_context());
    assert_eq!(outcome.actions, vec![ActionKind::Archive]);
    assert_eq!(outcome.matched, vec![RuleId::from_raw("a")]);
}
