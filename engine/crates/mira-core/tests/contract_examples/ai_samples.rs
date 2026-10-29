//! Valeurs d'exemple du domaine « assistant d'IA » : connexion, liste de modèles, fiche de fournisseur.

use mira_vocabulary::{
    AiConnection, AiConnectionId, AiModel, AiModelList, AiProviderDescriptor, AiProviderId,
    AiQuirks, AuthScheme, ContentKind, MaskingSettings, Timestamp, WireFormat,
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
