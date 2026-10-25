//! Exemples du domaine « modèles de messages » : modèles, rendu.

use mira_core::api::{Request, Response, TemplateRequest, TemplateResponse};
use mira_vocabulary::{
    ContactInfo, IdempotencyKey, MessageTemplate, RenderContext, RenderedTemplate, TemplateId,
};

use super::{Named, numbered};

/// Modèle d'exemple
fn sample_template() -> MessageTemplate {
    MessageTemplate {
        id: TemplateId::from_raw("template-1"),
        name: "Remerciement".to_string(),
        subject: Some("Merci {{contact.prénom}}".to_string()),
        body: "Bonjour {{contact.nom}}, merci. {{moi.nom}}".to_string(),
        created_at: 1_700_000_000_000,
        updated_at: 1_700_000_000_000,
    }
}

/// Contexte de rendu d'exemple
fn sample_context() -> RenderContext {
    RenderContext {
        contact: Some(ContactInfo {
            name: Some("Durand".to_string()),
            first_name: Some("Léa".to_string()),
            email: Some("lea@example.org".to_string()),
        }),
        account_name: Some("Noah".to_string()),
        account_email: Some("noah@example.org".to_string()),
        subject: Some("Devis".to_string()),
        date: Some("25/10/2026".to_string()),
        time: Some("14:30".to_string()),
    }
}

/// Les requêtes d'écriture : création, mise à jour, suppression
fn write_requests() -> Vec<TemplateRequest> {
    let id = TemplateId::from_raw("template-1");
    vec![
        TemplateRequest::CreateTemplate {
            name: "Remerciement".to_string(),
            subject: Some("Merci".to_string()),
            body: "Bonjour {{contact.nom}}".to_string(),
            idempotency_key: IdempotencyKey("key-1".to_string()),
        },
        TemplateRequest::UpdateTemplate {
            template: id.clone(),
            name: "Remerciement".to_string(),
            subject: None,
            body: "Bonjour".to_string(),
        },
        TemplateRequest::DeleteTemplate { template: id },
    ]
}

/// Une requête par opération de `TemplateRequest`
pub fn requests() -> Vec<Named<Request>> {
    let id = TemplateId::from_raw("template-1");
    let mut items = vec![
        TemplateRequest::ListTemplates,
        TemplateRequest::GetTemplate {
            template: id.clone(),
        },
    ];
    items.extend(write_requests());
    items.push(TemplateRequest::RenderTemplate {
        template: id,
        context: sample_context(),
    });
    let ops: Vec<&str> = items.iter().map(op_name).collect();
    numbered("template", &ops)
        .into_iter()
        .zip(items.into_iter().map(Request::Template))
        .collect()
}

/// Une réponse par forme de `TemplateResponse`
pub fn responses() -> Vec<Named<Response>> {
    let rendered = RenderedTemplate {
        subject: Some("Merci Léa".to_string()),
        body: "Bonjour Durand, merci. {{moi.nom}}".to_string(),
        unresolved: vec!["moi.nom".to_string()],
    };
    let items = [
        (
            "template.template",
            TemplateResponse::Template(sample_template()),
        ),
        (
            "template.templates",
            TemplateResponse::Templates(vec![sample_template()]),
        ),
        ("template.rendered", TemplateResponse::Rendered(rendered)),
        ("template.done", TemplateResponse::Done),
    ];
    let wrap =
        |(name, inner): (&str, TemplateResponse)| (name.to_string(), Response::Template(inner));
    items.into_iter().map(wrap).collect()
}

/// Nom de l'opération, par `match` exhaustif
pub fn op_name(request: &TemplateRequest) -> &'static str {
    match request {
        TemplateRequest::ListTemplates => "list_templates",
        TemplateRequest::GetTemplate { .. } => "get_template",
        TemplateRequest::CreateTemplate { .. } => "create_template",
        TemplateRequest::UpdateTemplate { .. } => "update_template",
        TemplateRequest::DeleteTemplate { .. } => "delete_template",
        TemplateRequest::RenderTemplate { .. } => "render_template",
    }
}
