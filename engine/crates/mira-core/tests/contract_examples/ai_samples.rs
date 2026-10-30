//! Valeurs d'exemple du domaine « assistant d'IA » : connexion, liste de modèles, fiche de fournisseur.

use mira_vocabulary::{
    AiConnection, AiConnectionId, AiJournalEntry, AiModel, AiModelList, AiProviderDescriptor,
    AiProviderId, AiQuirks, AuthScheme, ContentKind, MaskingSettings, OperationId, Timestamp,
    WireFormat,
};

/// Connexion d'exemple, avec liste de modèles gardée
pub fn sample_connection() -> AiConnection {
    AiConnection {
        id: AiConnectionId::from_raw("ai-1"),
        provider: AiProviderId("generic".to_string()),
        label: "Mon service".to_string(),
        base_url: Some("https://ia.example.org/v1".to_string()),
        default_model: Some("model-a".to_string()),
        is_default: true,
        has_key: true,
        consent_at: Some(Timestamp::from_millis(1_700_000_000_000)),
        models: Some(sample_models()),
        allowed_kinds: vec![ContentKind::MailMessage],
        masking: MaskingSettings::remote_default(),
    }
}

/// Liste de modèles d'exemple
pub fn sample_models() -> AiModelList {
    AiModelList {
        models: vec![AiModel {
            id: "model-a".to_string(),
            created_at: Some(1_690_000_000_000),
        }],
        fetched_at: Timestamp::from_millis(1_700_000_000_000),
    }
}

/// Fiche de fournisseur d'exemple
pub fn sample_provider() -> AiProviderDescriptor {
    AiProviderDescriptor {
        id: AiProviderId("generic".to_string()),
        display_name: "Service compatible".to_string(),
        base_url: "https://ia.example.org/v1".to_string(),
        auth: AuthScheme::Bearer,
        format: WireFormat::OpenAiChat,
        models_path: "/models".to_string(),
        model_exclusions: vec!["embed".to_string()],
        static_headers: Vec::new(),
        quirks: AiQuirks::default(),
        base_url_editable: true,
    }
}

/// Ligne de journal d'exemple : une génération terminée, sans aucun texte
pub fn sample_journal() -> AiJournalEntry {
    AiJournalEntry {
        id: "journal-1".to_string(),
        at: Timestamp::from_millis(1_700_000_000_000),
        operation: OperationId::from_raw("op-1"),
        connection: AiConnectionId::from_raw("ai-1"),
        provider: AiProviderId("generic".to_string()),
        model: "model-a".to_string(),
        local: false,
        kinds: vec![ContentKind::MailMessage],
        parts: 1,
        chars: 1234,
        masked: 2,
        input_tokens: Some(321),
        output_tokens: Some(45),
        outcome: "end_turn".to_string(),
    }
}
