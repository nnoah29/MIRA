#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! La fiche du fournisseur « chatgpt » annonce l'API Responses et la connexion de l'abonné, sans secret.

use mira_ai_chatgpt::ChatGptAiProvider;
use mira_vocabulary::{AiProvider, AiProviderId, AuthScheme, WireFormat};

/// La fiche reprend les valeurs de la documentation OpenAI
#[test]
fn chatgpt_descriptor_has_expected_values() {
    let d = ChatGptAiProvider.descriptor();
    assert_eq!(d.id, AiProviderId("chatgpt".to_string()));
    assert_eq!(d.display_name, "ChatGPT");
    assert_eq!(d.base_url, "https://api.openai.com/v1");
    assert_eq!(d.auth, AuthScheme::Bearer);
    assert_eq!(d.format, WireFormat::OpenAiResponses);
    assert_eq!(d.models_path, "/models");
    assert!(!d.base_url_editable);
    assert!(d.supports_tools);
    assert!(d.key_login.is_none());
}

/// La connexion de l'abonné porte les adresses et toutes les portées documentées
#[test]
fn chatgpt_descriptor_declares_the_session_login() {
    let login = ChatGptAiProvider.descriptor().session_login.unwrap();
    assert_eq!(
        login.authorize_url,
        "https://auth.openai.com/api/accounts/authorize"
    );
    assert_eq!(
        login.token_url,
        "https://auth.openai.com/api/accounts/oauth/token"
    );
    assert_eq!(login.resource, "https://api.openai.com/v1");
    assert_eq!(login.agent_name, "MIRA");
    assert_eq!(login.required_scope, "chatgpt.tokens.use.direct");
    let expected = [
        "openid",
        "profile",
        "email",
        "offline_access",
        "resource.invoke",
        "chatgpt.tokens.use.direct",
    ];
    assert_eq!(login.scopes, expected);
}

/// Aucun champ ne porte de secret ni l'identifiant du client Codex
#[test]
fn chatgpt_descriptor_has_no_secret_and_no_codex_client() {
    let text = format!("{:?}", ChatGptAiProvider.descriptor()).to_lowercase();
    for marker in ["app_emoamee", "backend-api", "sk-", "refresh_token"] {
        assert!(!text.contains(marker), "{text} contient {marker}");
    }
}
