//! Expressions régulières : valides, invalides, trop longues ou trop coûteuses.

use crate::support_rules::probe::{cond, text};
use crate::world::{valid_rule, world};
use mira_core::rules::validate::{MAX_REGEX_LEN, validate_rule};
use mira_vocabulary::{ConditionTest, Rule, TextOp};

/// Règle valide dont la condition est une regex sur l'objet
fn with_regex(pattern: &str) -> Rule {
    let mut rule = valid_rule();
    rule.alternatives[0].conditions[0] = cond(ConditionTest::Subject {
        matcher: text(TextOp::Regex, pattern),
    });
    rule
}

/// Une regex correcte est acceptée
#[test]
fn valid_regex_is_accepted() {
    assert!(validate_rule(&with_regex("^fact(ure)?\\s+\\d+$"), &world()).is_ok());
}

/// Une regex invalide est refusée sans reprendre sa valeur dans le message
#[test]
fn invalid_regex_is_refused_without_leaking_value() {
    let error = validate_rule(&with_regex("SECRET-xyz[unclosed"), &world()).unwrap_err();
    assert!(!error.message.contains("SECRET-xyz"));
}

/// Une regex plus longue que le plafond est refusée
#[test]
fn too_long_regex_is_refused() {
    let pattern = "a".repeat(MAX_REGEX_LEN + 1);
    assert!(validate_rule(&with_regex(&pattern), &world()).is_err());
}

/// Une regex dont la compilation dépasse la limite de taille est refusée
#[test]
fn oversized_compiled_regex_is_refused() {
    assert!(validate_rule(&with_regex("((a{100}){100}){100}"), &world()).is_err());
}
