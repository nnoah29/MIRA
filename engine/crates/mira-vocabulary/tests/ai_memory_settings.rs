#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Réglages, statut et propositions de la mémoire de l'IA : formes JSON stables et valeurs par défaut.

use mira_vocabulary::{
    AiConnectionId, AiLocalRuntime, ChatDelta, ContentKind, MemoryEmbedder, MemoryKind,
    MemoryProposal, MemoryProposalAction, MemorySettings, MemoryStatus,
};
use serde_json::json;

/// Sans réglage, la mémoire est désactivée et calcule sur l'appareil
#[test]
fn memory_settings_default_to_disabled_local() {
    let settings = MemorySettings::default();
    assert!(!settings.enabled);
    assert_eq!(settings.embedder, MemoryEmbedder::Local);
}

/// L'embedder s'écrit avec une étiquette `kind` et se relit à l'identique
#[test]
fn memory_embedder_is_tagged_and_round_trips() {
    let local = serde_json::to_value(MemoryEmbedder::Local).unwrap();
    assert_eq!(local, json!({"kind": "local"}));
    let provider = MemoryEmbedder::Provider {
        connection: AiConnectionId::from_raw("c-1"),
        model: "text-embedding-3-small".into(),
    };
    let value = serde_json::to_value(&provider).unwrap();
    assert_eq!(value["kind"], "provider");
    assert_eq!(value["model"], "text-embedding-3-small");
    let back: MemoryEmbedder = serde_json::from_value(value).unwrap();
    assert_eq!(back, provider);
}

/// Le statut se relit à l'identique
#[test]
fn memory_status_round_trips() {
    let status = MemoryStatus {
        settings: MemorySettings::default(),
        model_installed: true,
        count: 3,
        stale: 1,
    };
    let back: MemoryStatus =
        serde_json::from_str(&serde_json::to_string(&status).unwrap()).unwrap();
    assert_eq!(back, status);
}

/// La proposition de souvenir voyage dans un delta de type `memory_proposed`
#[test]
fn memory_proposed_delta_has_snake_case_type() {
    let delta = ChatDelta::MemoryProposed {
        id: "call-1".into(),
        proposal: MemoryProposal {
            action: MemoryProposalAction::Remember,
            text: "Je signe « Noah »".into(),
            kind: MemoryKind::Preference,
            id: None,
        },
    };
    let value = serde_json::to_value(&delta).unwrap();
    assert_eq!(value["type"], "memory_proposed");
    assert_eq!(value["proposal"]["action"], "remember");
    assert_eq!(value["proposal"]["kind"], "preference");
    let back: ChatDelta = serde_json::from_value(value).unwrap();
    assert_eq!(back, delta);
}

/// Les souvenirs sont un type de contenu à part, nommé `memory`
#[test]
fn memory_is_a_content_kind() {
    assert_eq!(ContentKind::Memory.as_str(), "memory");
    assert_eq!(
        serde_json::to_string(&ContentKind::Memory).unwrap(),
        "\"memory\""
    );
}

/// Une fiche de runtime écrite avant les embeddings se relit sans modèle d'embeddings
#[test]
fn runtime_without_embedding_field_reads_as_none() {
    let old = json!({
        "server": [], "server_args": [], "reasoning_off_args": [],
        "health_path": "/health", "idle_stop_seconds": 1,
        "start_timeout_seconds": 1, "models": []
    });
    let runtime: AiLocalRuntime = serde_json::from_value(old).unwrap();
    assert!(runtime.embedding.is_none());
}
