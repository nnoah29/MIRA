//! Exemples du contrat pour le modèle local : quatre requêtes, une réponse d'état.

use mira_core::api::{AiRequest, AiResponse};
use mira_vocabulary::{
    LocalInstallInfo, LocalModelStatus, LocalRuntimeStatus, LocalTier, OperationId,
};

/// Les quatre requêtes du modèle local
pub fn requests() -> Vec<AiRequest> {
    vec![
        AiRequest::GetLocalRuntimeStatus,
        AiRequest::InstallLocalModel {
            model_id: "tiny-model".to_string(),
        },
        AiRequest::RemoveLocalModel {
            model_id: "tiny-model".to_string(),
        },
        AiRequest::StopLocalRuntime,
    ]
}

/// Réponse d'état : un modèle installé, un autre en cours d'installation, avertissement mémoire
pub fn response() -> AiResponse {
    AiResponse::LocalRuntime(LocalRuntimeStatus {
        supported: true,
        models: vec![LocalModelStatus {
            id: "tiny-model".to_string(),
            display_name: "Tiny model".to_string(),
            tier: LocalTier::Light,
            size: 2_147_023_008,
            license: "Apache-2.0".to_string(),
            supports_tools: false,
            ram_recommended_mib: 3500,
            installed: false,
            memory_warning: true,
        }],
        server_installed: false,
        running_model: None,
        memory_available_mib: Some(2048),
        installing: Some(LocalInstallInfo {
            model_id: "tiny-model".to_string(),
            operation: OperationId::from_raw("op-1"),
        }),
    })
}
