#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, dead_code)]
//! Validation des règles qui répondent avec un modèle : le modèle doit exister.

mod support_rules;
#[path = "rules_validate/world.rs"]
mod world;

use mira_core::rules::validate::{RuleStatus, rule_status, validate_rule};
use mira_vocabulary::{Rule, RuleAction, TemplateId};
use world::{valid_rule, with_action, world};

/// Règle valide dont l'unique action répond avec le modèle donné
fn replies_with(raw: &str) -> Rule {
    with_action(RuleAction::ReplyWithTemplate {
        template: TemplateId::from_raw(raw),
    })
}

/// Un modèle connu est accepté et la règle est valide
#[test]
fn known_template_is_accepted() {
    let rule = replies_with("tpl-1");
    assert!(validate_rule(&rule, &world()).is_ok());
    assert_eq!(rule_status(&rule, &world()), RuleStatus::Valid);
}

/// Un modèle inconnu est refusé à l'enregistrement
#[test]
fn unknown_template_is_refused() {
    assert!(validate_rule(&replies_with("zz"), &world()).is_err());
}

/// Une règle dont le modèle a été supprimé devient invalide, avec la raison
#[test]
fn deleted_template_makes_the_rule_invalid() {
    let rule = replies_with("tpl-1");
    let mut after_deletion = world();
    after_deletion.templates.clear();
    assert!(matches!(
        rule_status(&rule, &after_deletion),
        RuleStatus::Invalid { .. }
    ));
}

/// Répondre avec un modèle se combine avec une action de rangement
#[test]
fn reply_can_be_combined_with_other_actions() {
    let mut rule = valid_rule();
    rule.actions.push(RuleAction::ReplyWithTemplate {
        template: TemplateId::from_raw("tpl-1"),
    });
    assert!(validate_rule(&rule, &world()).is_ok());
}
