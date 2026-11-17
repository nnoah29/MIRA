//! Niveau de risque des requêtes : assistant d'IA.

use mira_vocabulary::ToolRisk;

use crate::api::contract::AiRequest;

/// Niveau de risque des requêtes du domaine ai : les lectures sont nommées, tout le reste est sensible.
pub(super) fn ai_risk(request: &AiRequest) -> ToolRisk {
    let read = matches!(
        request,
        AiRequest::ListAiProviders
            | AiRequest::ListAiConnections
            | AiRequest::GetAiHistoryEnabled
            | AiRequest::ListAiJournal { .. }
            | AiRequest::GetLocalRuntimeStatus
    );
    if read {
        ToolRisk::Read
    } else {
        ToolRisk::Sensitive
    }
}
