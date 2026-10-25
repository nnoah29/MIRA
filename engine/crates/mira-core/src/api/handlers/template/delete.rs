//! Suppression d'un modèle de message.

use mira_vocabulary::{MiraError, TemplateId};

use super::writing::{announce, done};
use crate::api::contract::Response;
use crate::api::engine::Engine;

/// Supprime le modèle ; inconnu : `not_found.template`
pub(crate) fn handle_delete_template(
    engine: &Engine,
    template: &TemplateId,
) -> Result<Response, MiraError> {
    engine.database.write(|tx| tx.delete_template(template))?;
    announce(engine)?;
    Ok(done())
}
