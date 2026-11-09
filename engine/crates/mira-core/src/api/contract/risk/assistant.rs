//! Niveau de risque des requêtes : assistant d'IA.

use mira_vocabulary::ToolRisk;

use crate::api::contract::AiRequest;

/// Niveau de risque des requêtes du domaine ai.
pub(super) fn ai_risk(request: &AiRequest) -> ToolRisk {
    match request {
        AiRequest::ListAiProviders
        | AiRequest::ListAiConnections
        | AiRequest::GetAiHistoryEnabled
        | AiRequest::ListAiJournal { .. } => ToolRisk::Read,
        AiRequest::AddAiConnection { .. }
        | AiRequest::UpdateAiConnection { .. }
        | AiRequest::RemoveAiConnection { .. }
        | AiRequest::SetDefaultAiConnection { .. }
        | AiRequest::SetAiApiKey { .. }
        | AiRequest::ClearAiApiKey { .. }
        | AiRequest::StartAiKeyLogin { .. }
        | AiRequest::CompleteAiKeyLogin { .. }
        | AiRequest::CancelAiKeyLogin { .. }
        | AiRequest::TestAiConnection { .. }
        | AiRequest::ListAiModels { .. }
        | AiRequest::AcknowledgeAiConsent { .. }
        | AiRequest::SetAiAllowedContent { .. }
        | AiRequest::SetAiMasking { .. }
        | AiRequest::ClearAiJournal
        | AiRequest::SaveAiConversation { .. }
        | AiRequest::ListAiConversations { .. }
        | AiRequest::GetAiConversation { .. }
        | AiRequest::RenameAiConversation { .. }
        | AiRequest::DeleteAiConversation { .. }
        | AiRequest::ClearAiConversations
        | AiRequest::SetAiHistoryEnabled { .. } => ToolRisk::Sensitive,
    }
}
