//! Libellés locaux visés par une règle (conditions et actions) : ils doivent exister.

use mira_vocabulary::{ConditionTest, LocalLabelId, MiraError, Rule, RuleAction};

use super::world::RuleWorld;
use crate::actions::errors::input::invalid_parameter;

/// Libellé nommé par la condition, s'il y en a un.
fn condition_label(test: &ConditionTest) -> Option<&LocalLabelId> {
    match test {
        ConditionTest::HasLocalLabel { label } => Some(label),
        _ => None,
    }
}

/// Libellé nommé par l'action, s'il y en a un.
fn action_label(action: &RuleAction) -> Option<&LocalLabelId> {
    match action {
        RuleAction::AddLocalLabel { label } | RuleAction::RemoveLocalLabel { label } => Some(label),
        _ => None,
    }
}

/// Refuse une règle dont une condition ou une action vise un libellé local inexistant.
pub(super) fn check_local_labels(rule: &Rule, world: &dyn RuleWorld) -> Result<(), MiraError> {
    let in_conditions = rule
        .alternatives
        .iter()
        .flat_map(|group| group.conditions.iter())
        .filter_map(|condition| condition_label(&condition.test));
    let in_actions = rule.actions.iter().filter_map(action_label);
    if in_conditions
        .chain(in_actions)
        .any(|label| !world.local_label_exists(label))
    {
        return Err(invalid_parameter("un libellé local visé n'existe plus"));
    }
    Ok(())
}
