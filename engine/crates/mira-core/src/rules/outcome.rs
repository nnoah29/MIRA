//! Résultat de l'évaluation des règles sur un message.

use mira_vocabulary::{ActionKind, LocalLabelId, RuleId};

/// Pose (`on`) ou retrait d'un libellé local sur la conversation du message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LabelChange {
    pub label: LocalLabelId,
    pub on: bool,
}

/// Actions à soumettre, changements de libellés, silence demandé et règles déclenchées, dans l'ordre.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RuleOutcome {
    pub actions: Vec<ActionKind>,
    pub label_changes: Vec<LabelChange>,
    pub silenced: bool,
    pub matched: Vec<RuleId>,
}
