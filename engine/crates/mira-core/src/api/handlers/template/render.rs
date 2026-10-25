//! Rendu d'un modèle de message dans un contexte fourni par le client.

use mira_vocabulary::{MiraError, RenderContext, TemplateId};

use crate::api::contract::{Response, TemplateResponse};
use crate::api::engine::Engine;
use crate::storage::repositories::message_templates::require_template;
use crate::templates::render_template;

/// Rend le modèle sans rien écrire ; les variables sans valeur restent visibles ; inconnu : `not_found.template`
pub(crate) fn handle_render_template(
    engine: &Engine,
    template: &TemplateId,
    context: &RenderContext,
) -> Result<Response, MiraError> {
    let found = engine.database.read(|tx| require_template(tx, template))?;
    let rendered = render_template(&found, context);
    Ok(Response::Template(TemplateResponse::Rendered(rendered)))
}
