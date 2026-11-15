//! Noms d'opération des requêtes d'historique et de modèle local, pour les exemples du contrat.

use mira_core::api::AiRequest;

/// Nom de l'opération pour l'historique des conversations et le modèle local
pub fn op_name(request: &AiRequest) -> &'static str {
    match request {
        AiRequest::SaveAiConversation { .. } => "save_ai_conversation",
        AiRequest::ListAiConversations { .. } => "list_ai_conversations",
        AiRequest::GetAiConversation { .. } => "get_ai_conversation",
        AiRequest::RenameAiConversation { .. } => "rename_ai_conversation",
        AiRequest::DeleteAiConversation { .. } => "delete_ai_conversation",
        AiRequest::ClearAiConversations => "clear_ai_conversations",
        AiRequest::SetAiHistoryEnabled { .. } => "set_ai_history_enabled",
        AiRequest::GetAiHistoryEnabled => "get_ai_history_enabled",
        AiRequest::GetLocalRuntimeStatus => "get_local_runtime_status",
        AiRequest::InstallLocalModel { .. } => "install_local_model",
        AiRequest::RemoveLocalModel { .. } => "remove_local_model",
        AiRequest::StopLocalRuntime => "stop_local_runtime",
        AiRequest::InstallMemoryModel => "install_memory_model",
        AiRequest::GetMemoryStatus => "get_memory_status",
        AiRequest::SetMemorySettings { .. } => "set_memory_settings",
        AiRequest::ListMemories { .. } => "list_memories",
        AiRequest::SearchMemories { .. } => "search_memories",
        AiRequest::RememberMemory { .. } => "remember_memory",
        AiRequest::UpdateMemory { .. } => "update_memory",
        AiRequest::ForgetMemory { .. } => "forget_memory",
        AiRequest::ForgetAllMemories => "forget_all_memories",
        AiRequest::ReindexMemories => "reindex_memories",
        _ => "unknown",
    }
}
