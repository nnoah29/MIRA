//! Lecture du résultat d'évaluation d'une règle à une condition.

pub use super::builders::{cond, not, rule, scoped, text};
use mira_core::rules::context::MessageContext;
use mira_core::rules::evaluate::evaluate;
use mira_vocabulary::{Condition, ConditionTest, RuleAction};

/// Vrai si une règle à la seule condition donnée se déclenche sur le contexte
pub fn holds(condition: Condition, ctx: &MessageContext) -> bool {
    let rules = [rule(
        "probe",
        vec![vec![condition]],
        vec![RuleAction::Archive],
    )];
    !evaluate(&rules, ctx).matched.is_empty()
}

/// Condition vraie sur le contexte de référence
pub fn yes() -> Condition {
    cond(ConditionTest::HasAttachment)
}

/// Condition fausse sur le contexte de référence
pub fn no() -> Condition {
    cond(ConditionTest::IsRead)
}

/// Vrai si le test seul (non inversé) se vérifie
pub fn test_holds(test: ConditionTest, ctx: &MessageContext) -> bool {
    holds(cond(test), ctx)
}
