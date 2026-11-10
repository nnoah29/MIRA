//! Fiche du fournisseur d'IA local : serveur compatible OpenAI Chat sur la boucle locale.

use mira_vocabulary::{AiProviderDescriptor, AiProviderId, AiQuirks, AuthScheme, WireFormat};

use crate::pinned::runtime;

/// Identifiant du fournisseur dans le catalogue.
pub const PROVIDER_ID: &str = "local";

/// Construit la fiche statique du fournisseur.
pub fn descriptor() -> AiProviderDescriptor {
    AiProviderDescriptor {
        id: AiProviderId(PROVIDER_ID.to_string()),
        display_name: "Modèle local".to_string(),
        base_url: "http://127.0.0.1".to_string(),
        auth: AuthScheme::Bearer,
        format: WireFormat::OpenAiChat,
        models_path: "/v1/models".to_string(),
        model_exclusions: Vec::new(),
        static_headers: Vec::new(),
        quirks: AiQuirks::default(),
        base_url_editable: false,
        supports_tools: false,
        key_login: None,
        session_login: None,
        local_runtime: Some(runtime()),
    }
}
