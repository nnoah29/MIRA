//! Réponses du domaine « assistant d'IA » : fournisseurs, connexions, modèles.

use mira_vocabulary::{AiConnection, AiModelList, AiProviderDescriptor};
use serde::{Deserialize, Serialize};

/// Réponse à une requête du domaine assistant d'IA ; jamais de clé.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum AiResponse {
    Providers(Vec<AiProviderDescriptor>),
    Connections(Vec<AiConnection>),
    Connection(AiConnection),
    Models(AiModelList),
    Done,
}
