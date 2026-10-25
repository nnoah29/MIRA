//! Mise à jour d'un modèle de message.

use mira_vocabulary::{MiraError, TemplateId};

use super::writing::{announce, single};
use crate::api::contract::Response;
use crate::api::engine::Engine;
use crate::storage::repositories::message_templates::require_template;

/// Remplace nom, objet et corps ; champ invalide ou nom pris : refusé sans écrire ; inconnu : `not_found.template`
pub(crate) fn handle_update_template(
    engine: &Engine,
    template: &TemplateId,
    name: &str,
    subject: Option<&str>,
    body: &str,
) -> Result<Response, MiraError> {
    let now = engine.config.clock.now().as_millis();
    let updated = engine.database.write(|tx| {
        tx.update_template(template, name, subject, body, now)?;
        require_template(tx, template)
    })?;
    announce(engine)?;
    Ok(single(updated))
}
