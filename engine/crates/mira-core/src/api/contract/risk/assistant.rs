//! Niveau de risque des requêtes : assistant d'IA.

use mira_vocabulary::ToolRisk;

use crate::api::contract::AiRequest;

/// Niveau de risque des requêtes du domaine ai.
pub(super) fn ai_risk(request: &AiRequest) -> ToolRisk {
    match request {
        AiRequest::ListAiProviders
        | AiRequest::ListAiConnections
        | AiRequest::ListAiJournal { .. } => ToolRisk::Read,
        AiRequest::AddAiConnection { .. }
        | AiRequest::UpdateAiConnection { .. }
        | AiRequest::RemoveAiConnection { .. }
        | AiRequest::SetDefaultAiConnection { .. }
        | AiRequest::SetAiApiKey { .. }
        | AiRequest::ClearAiApiKey { .. }
        | AiRequest::TestAiConnection { .. }
        | AiRequest::ListAiModels { .. }
        | AiRequest::AcknowledgeAiConsent { .. }
        | AiRequest::SetAiAllowedContent { .. }
        | AiRequest::SetAiMasking { .. }
        | AiRequest::ClearAiJournal => ToolRisk::Sensitive,
    }
}
