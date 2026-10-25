//! Appels du domaine « template » par la façade et lecture de leurs réponses.

use mira_core::api::{Engine, Request, Response, TemplateRequest, TemplateResponse};
use mira_vocabulary::{
    EventKind, IdempotencyKey, MessageTemplate, RenderContext, RenderedTemplate, TemplateId,
};

use crate::support_api::queries::event_kinds;

/// Exécute une requête du domaine et rend la réponse du domaine, ou le code d'erreur
pub async fn call(engine: &Engine, request: TemplateRequest) -> Result<TemplateResponse, String> {
    match engine.execute(Request::Template(request)).await {
        Ok(Response::Template(answer)) => Ok(answer),
        Ok(other) => panic!("réponse inattendue : {other:?}"),
        Err(error) => Err(error.code),
    }
}

/// Crée un modèle sans objet sous cette clé et le rend, ou le code d'erreur
pub async fn create(
    engine: &Engine,
    name: &str,
    body: &str,
    key: &str,
) -> Result<MessageTemplate, String> {
    let request = TemplateRequest::CreateTemplate {
        name: name.to_string(),
        subject: None,
        body: body.to_string(),
        idempotency_key: IdempotencyKey(key.to_string()),
    };
    match call(engine, request).await? {
        TemplateResponse::Template(template) => Ok(template),
        other => panic!("réponse inattendue : {other:?}"),
    }
}

/// Modèles listés par le moteur
pub async fn templates(engine: &Engine) -> Vec<MessageTemplate> {
    match call(engine, TemplateRequest::ListTemplates).await.unwrap() {
        TemplateResponse::Templates(found) => found,
        other => panic!("réponse inattendue : {other:?}"),
    }
}

/// Rend le modèle dans ce contexte, ou le code d'erreur
pub async fn render(
    engine: &Engine,
    template: &TemplateId,
    context: RenderContext,
) -> Result<RenderedTemplate, String> {
    let request = TemplateRequest::RenderTemplate {
        template: template.clone(),
        context,
    };
    match call(engine, request).await? {
        TemplateResponse::Rendered(rendered) => Ok(rendered),
        other => panic!("réponse inattendue : {other:?}"),
    }
}

/// Nombre d'événements `TemplatesChanged` émis depuis le début
pub async fn changed_count(engine: &Engine) -> usize {
    let kinds = event_kinds(engine).await;
    kinds
        .iter()
        .filter(|kind| matches!(kind, EventKind::TemplatesChanged))
        .count()
}
