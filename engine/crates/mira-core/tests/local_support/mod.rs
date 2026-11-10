#![allow(dead_code)]
//! Outils des tests du modèle local : fiche de runtime de test, sans nommer de vrai fournisseur.

use mira_vocabulary::{
    AiLocalModel, AiLocalRuntime, AiLocalServerBuild, AiProviderDescriptor, AiProviderId, AiQuirks,
    AuthScheme, LocalTier, WireFormat,
};

/// Modèle de test de ce palier ; fichier et somme à compléter par le test qui télécharge
pub fn test_model(id: &str, tier: LocalTier, tools: bool) -> AiLocalModel {
    AiLocalModel {
        id: id.to_string(),
        display_name: format!("Modèle {id}"),
        tier,
        url: format!("https://models.test/{id}.bin"),
        file_name: format!("{id}.bin"),
        sha256: "0".repeat(64),
        size: 0,
        license: "Apache-2.0".to_string(),
        ram_recommended_mib: 1000,
        ctx_size: 4096,
        supports_tools: tools,
        reasoning: false,
    }
}

/// Fiche de runtime de test : un modèle sans outils (`mini`) et un avec outils (`big`)
pub fn test_runtime() -> AiLocalRuntime {
    AiLocalRuntime {
        server: Vec::<AiLocalServerBuild>::new(),
        server_args: ["--model", "{model}", "--host", "{host}", "--port", "{port}"]
            .iter()
            .map(|a| (*a).to_string())
            .collect(),
        reasoning_off_args: Vec::new(),
        health_path: "/health".to_string(),
        idle_stop_seconds: 300,
        start_timeout_seconds: 120,
        models: vec![
            test_model("mini", LocalTier::Mini, false),
            test_model("big", LocalTier::Advanced, true),
        ],
    }
}

/// Fiche d'un fournisseur de test, avec ou sans runtime local
pub fn descriptor(
    id: &str,
    supports_tools: bool,
    local_runtime: Option<AiLocalRuntime>,
) -> AiProviderDescriptor {
    AiProviderDescriptor {
        id: AiProviderId(id.to_string()),
        display_name: id.to_string(),
        base_url: "http://127.0.0.1".to_string(),
        auth: AuthScheme::Bearer,
        format: WireFormat::OpenAiChat,
        models_path: "/v1/models".to_string(),
        model_exclusions: Vec::new(),
        static_headers: Vec::new(),
        quirks: AiQuirks::default(),
        base_url_editable: false,
        supports_tools,
        key_login: None,
        session_login: None,
        local_runtime,
    }
}
