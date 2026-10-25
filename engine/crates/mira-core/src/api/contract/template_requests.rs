//! Requêtes du domaine « modèles de messages » : une variante, une opération, un handler.

use mira_vocabulary::{IdempotencyKey, RenderContext, TemplateId};
use serde::{Deserialize, Serialize};

/// Opérations sur les modèles de messages ; la création attribue l'identifiant.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum TemplateRequest {
    ListTemplates,
    GetTemplate {
        template: TemplateId,
    },
    CreateTemplate {
        name: String,
        subject: Option<String>,
        body: String,
        idempotency_key: IdempotencyKey,
    },
    UpdateTemplate {
        template: TemplateId,
        name: String,
        subject: Option<String>,
        body: String,
    },
    DeleteTemplate {
        template: TemplateId,
    },
    RenderTemplate {
        template: TemplateId,
        context: RenderContext,
    },
}
