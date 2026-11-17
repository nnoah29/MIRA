//! Fiche du fournisseur d'IA ChatGPT : API Responses, jetons de l'abonné obtenus par navigateur.

use mira_vocabulary::{
    AiProviderDescriptor, AiProviderId, AiQuirks, AiSessionLogin, AuthScheme, WireFormat,
};

/// Identifiant du fournisseur dans le catalogue.
pub const PROVIDER_ID: &str = "chatgpt";

/// Construit la fiche statique du fournisseur.
pub fn descriptor() -> AiProviderDescriptor {
    AiProviderDescriptor {
        id: AiProviderId(PROVIDER_ID.to_string()),
        display_name: "ChatGPT".to_string(),
        base_url: "https://api.openai.com/v1".to_string(),
        auth: AuthScheme::Bearer,
        format: WireFormat::OpenAiResponses,
        models_path: "/models".to_string(),
        model_exclusions: Vec::new(),
        static_headers: Vec::new(),
        quirks: AiQuirks::default(),
        base_url_editable: false,
        supports_tools: true,
        key_login: None,
        session_login: Some(session_login()),
        local_runtime: None,
    }
}

/// Adresses et portées de la connexion de l'abonné (partage de jetons).
fn session_login() -> AiSessionLogin {
    let scopes = [
        "openid",
        "profile",
        "email",
        "offline_access",
        "resource.invoke",
        "chatgpt.tokens.use.direct",
    ];
    AiSessionLogin {
        authorize_url: "https://auth.openai.com/api/accounts/authorize".to_string(),
        token_url: "https://auth.openai.com/api/accounts/oauth/token".to_string(),
        resource: "https://api.openai.com/v1".to_string(),
        scopes: scopes.iter().map(|scope| (*scope).to_string()).collect(),
        required_scope: "chatgpt.tokens.use.direct".to_string(),
        agent_name: "MIRA".to_string(),
    }
}
