//! Exemples du contrat pour la mémoire de l'IA : requêtes et réponses.

use mira_core::api::{AiRequest, AiResponse};
use mira_vocabulary::{
    MemoryEmbedder, MemoryKind, MemoryMatch, MemoryRecord, MemorySettings, MemorySource,
    MemoryStatus, Timestamp,
};

/// Souvenir d'exemple
fn sample_record() -> MemoryRecord {
    MemoryRecord {
        id: "mem-1".to_string(),
        text: "Je signe mes mails « Noah ».".to_string(),
        kind: MemoryKind::Preference,
        source: MemorySource::User,
        created_at: Timestamp::from_millis(1_700_000_000_000),
        model_id: Some("embedding-model-1".to_string()),
    }
}

/// Réglages, installation et lectures
fn setup_and_read_requests() -> Vec<AiRequest> {
    vec![
        AiRequest::GetMemoryStatus,
        AiRequest::SetMemorySettings {
            settings: MemorySettings {
                enabled: true,
                embedder: MemoryEmbedder::Local,
            },
        },
        AiRequest::InstallMemoryModel,
        AiRequest::ListMemories {
            limit: 50,
            after: Some("mem-1".to_string()),
        },
        AiRequest::SearchMemories {
            query: "comment je signe".to_string(),
            limit: 5,
        },
    ]
}

/// Écritures : retenir, modifier, oublier, tout oublier, réindexer
fn write_requests() -> Vec<AiRequest> {
    vec![
        AiRequest::RememberMemory {
            text: "Je signe mes mails « Noah ».".to_string(),
            kind: MemoryKind::Preference,
        },
        AiRequest::UpdateMemory {
            id: "mem-1".to_string(),
            text: "Je signe « Noah T. ».".to_string(),
        },
        AiRequest::ForgetMemory {
            id: "mem-1".to_string(),
        },
        AiRequest::ForgetAllMemories,
        AiRequest::ReindexMemories,
        AiRequest::GetAboutMe,
        AiRequest::SetAboutMe {
            text: "Je suis médecin. Je signe « Noah ».".to_string(),
        },
    ]
}

/// Une requête par opération de la mémoire
pub fn requests() -> Vec<AiRequest> {
    let mut items = setup_and_read_requests();
    items.extend(write_requests());
    items
}

/// Les réponses de la mémoire, nommées d'après leur forme
pub fn responses() -> Vec<(&'static str, AiResponse)> {
    let status = MemoryStatus {
        settings: MemorySettings::default(),
        model_installed: true,
        count: 12,
        stale: 2,
    };
    vec![
        ("ai.memory_status", AiResponse::MemoryStatus(status)),
        ("ai.memories", AiResponse::Memories(vec![sample_record()])),
        (
            "ai.memory_matches",
            AiResponse::MemoryMatches(vec![MemoryMatch {
                record: sample_record(),
                score_permille: 874,
            }]),
        ),
        ("ai.memory", AiResponse::Memory(sample_record())),
        ("ai.memory_forgotten", AiResponse::MemoryForgotten(12)),
        (
            "ai.about_me",
            AiResponse::AboutMe("Je suis médecin.".to_string()),
        ),
        (
            "ai.memory_reindexed",
            AiResponse::MemoryReindexed { done: 2, failed: 0 },
        ),
    ]
}
