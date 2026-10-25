//! Domaine « modèles de messages » : un fichier par opération, écritures contrôlées, création idempotente.

mod create;
mod delete;
mod get;
mod idempotency;
mod list;
mod render;
mod update;
mod writing;

use mira_vocabulary::MiraError;

use crate::api::contract::{Response, TemplateRequest, TemplateRequest as R};
use crate::api::engine::Engine;

/// Résultat d'une requête du domaine
type Answer = Result<Response, MiraError>;

/// Route une requête du domaine modèles de messages
pub(crate) async fn route(engine: &Engine, request: TemplateRequest) -> Answer {
    match request {
        R::ListTemplates => list::handle_list_templates(engine),
        R::GetTemplate { template } => get::handle_get_template(engine, &template),
        R::CreateTemplate {
            name,
            subject,
            body,
            idempotency_key,
        } => create::handle_create_template(
            engine,
            &name,
            subject.as_deref(),
            &body,
            idempotency_key,
        ),
        R::UpdateTemplate {
            template,
            name,
            subject,
            body,
        } => update::handle_update_template(engine, &template, &name, subject.as_deref(), &body),
        R::DeleteTemplate { template } => delete::handle_delete_template(engine, &template),
        R::RenderTemplate { template, context } => {
            render::handle_render_template(engine, &template, &context)
        }
    }
}
