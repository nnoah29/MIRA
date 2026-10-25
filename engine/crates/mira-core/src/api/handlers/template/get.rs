//! Lecture d'un modèle de message.

use mira_vocabulary::{MiraError, TemplateId};

use super::writing::single;
use crate::api::contract::Response;
use crate::api::engine::Engine;
use crate::storage::repositories::message_templates::require_template;

/// Rend le modèle ; inconnu : `not_found.template`
pub(crate) fn handle_get_template(
    engine: &Engine,
    template: &TemplateId,
) -> Result<Response, MiraError> {
    let found = engine.database.read(|tx| require_template(tx, template))?;
    Ok(single(found))
}
