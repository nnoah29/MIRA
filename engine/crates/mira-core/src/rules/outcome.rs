//! Résultat de l'évaluation des règles sur un message.

use mira_vocabulary::{ActionKind, RuleId};

/// Actions à soumettre, silence demandé et règles déclenchées, dans l'ordre.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RuleOutcome {
    pub actions: Vec<ActionKind>,
    pub silenced: bool,
    pub matched: Vec<RuleId>,
}
