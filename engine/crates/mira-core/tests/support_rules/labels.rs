//! Libellés locaux dans les règles de test.

use mira_core::rules::outcome::LabelChange;
use mira_vocabulary::{ConditionTest, LocalLabelId, Rule, RuleAction};

use super::builders::{cond, rule};

/// Identifiant de libellé de test
pub fn label(raw: &str) -> LocalLabelId {
    LocalLabelId::from_raw(raw)
}

/// Règle qui teste la présence du libellé
pub fn has(id: &str, wanted: &str, actions: Vec<RuleAction>) -> Rule {
    let test = ConditionTest::HasLocalLabel {
        label: label(wanted),
    };
    rule(id, vec![vec![cond(test)]], actions)
}

/// Changement de libellé attendu
pub fn change(raw: &str, on: bool) -> LabelChange {
    LabelChange {
        label: label(raw),
        on,
    }
}
