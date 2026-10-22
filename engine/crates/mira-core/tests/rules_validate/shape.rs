//! Contrôles de forme : nombre de valeurs d'un filtre, longueur du nom, et `validate_shape` sans le monde.

use crate::support_rules::probe::cond;
use crate::world::{valid_rule, world};
use mira_core::rules::validate::{
    MAX_NAME_LEN, MAX_REGEX_LEN, MAX_VALUES_PER_MATCHER, validate_rule, validate_shape,
};
use mira_vocabulary::{ConditionTest, MailboxId, Rule, RuleAction, TextMatcher, TextOp};

/// Règle valide dont le filtre d'objet porte `count` valeurs « in_list »
fn with_values(count: usize) -> Rule {
    let mut rule = valid_rule();
    let values = (0..count).map(|n| format!("v{n}")).collect();
    rule.alternatives[0].conditions[0] = cond(ConditionTest::Subject {
        matcher: TextMatcher {
            op: TextOp::InList,
            values,
        },
    });
    rule
}

/// Un filtre avec plus de valeurs que le plafond est refusé, à l'enregistrement comme à l'aperçu
#[test]
fn too_many_values_in_a_filter_are_refused() {
    let rule = with_values(MAX_VALUES_PER_MATCHER + 1);
    let error = validate_rule(&rule, &world()).unwrap_err();
    assert_eq!(error.code, "invalid_input.parameter");
    assert!(validate_shape(&rule).is_err());
}

/// Un nom plus long que le plafond est refusé, à l'enregistrement comme à l'aperçu
#[test]
fn too_long_name_is_refused() {
    let mut rule = valid_rule();
    rule.name = "n".repeat(MAX_NAME_LEN + 1);
    let error = validate_rule(&rule, &world()).unwrap_err();
    assert_eq!(error.code, "invalid_input.parameter");
    assert!(validate_shape(&rule).is_err());
}

/// Le nombre de valeurs et la longueur du nom exactement au plafond sont acceptés
#[test]
fn values_and_name_at_exact_caps_are_accepted() {
    let mut rule = with_values(MAX_VALUES_PER_MATCHER);
    rule.name = "é".repeat(MAX_NAME_LEN / 2);
    assert_eq!(rule.name.len(), MAX_NAME_LEN);
    assert!(validate_rule(&rule, &world()).is_ok());
    assert!(validate_shape(&rule).is_ok());
}

/// `validate_shape` ignore le monde et l'absence d'action, mais refuse une regex trop longue
#[test]
fn validate_shape_checks_form_only() {
    let mut draft = valid_rule();
    draft.actions = vec![RuleAction::MoveTo {
        mailbox: MailboxId::from_raw("inconnue"),
    }];
    assert!(validate_shape(&draft).is_ok());
    draft.actions.clear();
    assert!(validate_shape(&draft).is_ok());
    draft.alternatives[0].conditions[0] = cond(ConditionTest::Subject {
        matcher: TextMatcher {
            op: TextOp::Regex,
            values: vec!["a".repeat(MAX_REGEX_LEN + 1)],
        },
    });
    assert!(validate_shape(&draft).is_err());
}
