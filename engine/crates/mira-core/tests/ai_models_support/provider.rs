//! Fournisseur d'IA de test : aucun nom de fournisseur réel dans le cœur.

use mira_vocabulary::{
    AiProvider, AiProviderDescriptor, AiProviderId, AiQuirks, AuthScheme, WireFormat,
};

/// Fournisseur de test dont l'identifiant, le format et les exclusions se règlent.
#[derive(Debug, Clone)]
pub struct TestProvider {
    pub id: String,
    pub base_url: String,
    pub auth: AuthScheme,
    pub format: WireFormat,
    pub exclusions: Vec<String>,
    pub editable: bool,
}

impl TestProvider {
    /// Fournisseur au format OpenAI, clé en Bearer, sans exclusion
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            base_url: "https://api.provider.test/v1".to_string(),
            auth: AuthScheme::Bearer,
            format: WireFormat::OpenAiChat,
            exclusions: Vec::new(),
            editable: true,
        }
    }
}

impl AiProvider for TestProvider {
    /// Fiche du fournisseur de test
    fn descriptor(&self) -> AiProviderDescriptor {
        AiProviderDescriptor {
            id: AiProviderId(self.id.clone()),
            display_name: format!("Fournisseur {}", self.id),
            base_url: self.base_url.clone(),
            auth: self.auth,
            format: self.format,
            models_path: "/models".to_string(),
            model_exclusions: self.exclusions.clone(),
            static_headers: Vec::new(),
            quirks: AiQuirks::default(),
            base_url_editable: self.editable,
            supports_tools: true,
        }
    }
}
