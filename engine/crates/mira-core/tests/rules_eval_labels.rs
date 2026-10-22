#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, dead_code)]
//! Libellés locaux dans les règles : condition, actions, ordre des changements, effet d'une règle sur la suivante.

mod support_rules;

use mira_core::rules::evaluate::evaluate;
use mira_vocabulary::{ActionKind, ConditionTest, Rule, RuleAction};
use support_rules::builders::{not, rule};
use support_rules::fixtures::base_context;
use support_rules::labels::{change, has, label};

/// « A le libellé » est vrai quand la conversation le porte, faux sinon, et s'inverse
#[test]
fn has_local_label_is_true_false_and_negated() {
    let mut ctx = base_context();
    ctx.thread_labels = vec![label("l1")];
    let fires = |r: Rule| evaluate(&[r], &ctx).matched.len();
    assert_eq!(fires(has("a", "l1", vec![RuleAction::Archive])), 1);
    assert_eq!(fires(has("b", "l2", vec![RuleAction::Archive])), 0);
    let absent = ConditionTest::HasLocalLabel { label: label("l2") };
    let negated = rule("c", vec![vec![not(absent)]], vec![RuleAction::Archive]);
    assert_eq!(fires(negated), 1);
}

/// Les actions de libellé produisent des changements dans l'ordre, sans action de file
#[test]
fn add_and_remove_are_listed_in_order() {
    let actions = vec![
        RuleAction::AddLocalLabel { label: label("l1") },
        RuleAction::RemoveLocalLabel { label: label("l2") },
    ];
    let always = has("r", "l0", actions);
    let mut ctx = base_context();
    ctx.thread_labels = vec![label("l0")];
    let outcome = evaluate(&[always], &ctx);
    assert_eq!(
        outcome.label_changes,
        vec![change("l1", true), change("l2", false)]
    );
    assert!(outcome.actions.is_empty());
}

/// Une règle voit l'effet de la précédente : le libellé posé ou retiré se lit aussitôt
#[test]
fn later_rule_sees_the_effect_of_an_earlier_one() {
    let mut ctx = base_context();
    ctx.thread_labels = vec![label("l0")];
    let adds = has(
        "r1",
        "l0",
        vec![RuleAction::AddLocalLabel { label: label("l1") }],
    );
    let reads = has("r2", "l1", vec![RuleAction::MarkRead { read: true }]);
    let outcome = evaluate(&[adds, reads], &ctx);
    assert_eq!(outcome.actions, vec![ActionKind::MarkRead { read: true }]);
    assert_eq!(outcome.label_changes, vec![change("l1", true)]);
    let removes = has(
        "r3",
        "l0",
        vec![RuleAction::RemoveLocalLabel { label: label("l0") }],
    );
    let gone = has("r4", "l0", vec![RuleAction::Trash]);
    assert!(evaluate(&[removes, gone], &ctx).actions.is_empty());
}

/// Sur un même libellé, les changements restent dans l'ordre des règles : le dernier l'emporte à l'application
#[test]
fn last_rule_wins_on_same_label() {
    let mut ctx = base_context();
    ctx.thread_labels = vec![label("l0")];
    let first = has(
        "r1",
        "l0",
        vec![RuleAction::AddLocalLabel { label: label("l1") }],
    );
    let second = has(
        "r2",
        "l0",
        vec![RuleAction::RemoveLocalLabel { label: label("l1") }],
    );
    let outcome = evaluate(&[first, second], &ctx);
    assert_eq!(
        outcome.label_changes,
        vec![change("l1", true), change("l1", false)]
    );
}
