//! Réponses du domaine « modèles de messages » : modèle, liste, rendu.

use mira_vocabulary::{MessageTemplate, RenderedTemplate};
use serde::{Deserialize, Serialize};

/// Réponse à une requête du domaine modèles de messages.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum TemplateResponse {
    Template(MessageTemplate),
    Templates(Vec<MessageTemplate>),
    Rendered(RenderedTemplate),
    Done,
}
