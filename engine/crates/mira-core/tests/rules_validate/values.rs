//! Valeurs de filtre vides : une liste vide ou une valeur vide (ou faite d'espaces) est refusée.

use crate::support_rules::probe::cond;
use crate::world::{valid_rule, world};
use mira_core::rules::validate::{validate_rule, validate_shape};
use mira_vocabulary::{ConditionTest, Rule, TextMatcher, TextOp};

/// Règle valide dont le filtre d'objet porte les valeurs données
fn with_subject_values(op: TextOp, values: &[&str]) -> Rule {
    let mut rule = valid_rule();
    rule.alternatives[0].conditions[0] = cond(ConditionTest::Subject {
        matcher: TextMatcher {
            op,
            values: values.iter().map(|v| v.to_string()).collect(),
        },
    });
    rule
}

/// Une valeur vide est refusée, à l'enregistrement comme à l'aperçu, avec un message statique
#[test]
fn empty_value_is_refused() {
    let rule = with_subject_values(TextOp::Contains, &[""]);
    let error = validate_rule(&rule, &world()).unwrap_err();
    assert_eq!(error.code, "invalid_input.parameter");
    assert_eq!(error.message, "un filtre a une valeur vide");
    assert!(validate_shape(&rule).is_err());
}

/// Une valeur faite d'espaces est refusée, même parmi des valeurs correctes, sans reprendre les valeurs
#[test]
fn blank_value_among_others_is_refused_without_leaking_values() {
    let rule = with_subject_values(TextOp::InList, &["SECRET-abc", "  \t "]);
    let error = validate_rule(&rule, &world()).unwrap_err();
    assert_eq!(error.message, "un filtre a une valeur vide");
    assert!(!error.message.contains("SECRET-abc"));
}

/// Un filtre sans aucune valeur est refusé, pour chaque opérateur
#[test]
fn empty_value_list_is_refused() {
    for op in [
        TextOp::Contains,
        TextOp::Equals,
        TextOp::InList,
        TextOp::Regex,
    ] {
        let rule = with_subject_values(op, &[]);
        let error = validate_rule(&rule, &world()).unwrap_err();
        assert_eq!(error.message, "un filtre a une valeur vide");
        assert!(validate_shape(&rule).is_err());
    }
}

/// Une valeur non vide entourée d'espaces reste acceptée
#[test]
fn padded_value_is_accepted() {
    let rule = with_subject_values(TextOp::Contains, &[" facture "]);
    assert!(validate_rule(&rule, &world()).is_ok());
}
