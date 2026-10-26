//! Modèles de message visés par les actions d'une règle : ils doivent exister.

use mira_vocabulary::{MiraError, Rule, RuleAction, TemplateId};

use super::world::RuleWorld;
use crate::actions::errors::input::invalid_parameter;

/// Modèle nommé par l'action, s'il y en a un.
fn action_template(action: &RuleAction) -> Option<&TemplateId> {
    match action {
        RuleAction::ReplyWithTemplate { template } => Some(template),
        _ => None,
    }
}

/// Refuse une règle dont une action répond avec un modèle inexistant.
pub(super) fn check_templates(rule: &Rule, world: &dyn RuleWorld) -> Result<(), MiraError> {
    if rule
        .actions
        .iter()
        .filter_map(action_template)
        .any(|template| !world.template_exists(template))
    {
        return Err(invalid_parameter("un modèle de message visé n'existe plus"));
    }
    Ok(())
}
