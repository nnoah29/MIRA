//! Annonce et réponses communes aux écritures de modèles de messages.

use mira_vocabulary::{EventKind, MessageTemplate, MiraError};

use crate::api::contract::{Response, TemplateResponse};
use crate::api::engine::Engine;
use crate::events::EventSink;

/// Annonce que les modèles ont changé
pub(super) fn announce(engine: &Engine) -> Result<(), MiraError> {
    engine.events.emit(EventKind::TemplatesChanged)?;
    Ok(())
}

/// Réponse portant un seul modèle
pub(super) fn single(template: MessageTemplate) -> Response {
    Response::Template(TemplateResponse::Template(template))
}

/// Réponse sans contenu
pub(super) fn done() -> Response {
    Response::Template(TemplateResponse::Done)
}
