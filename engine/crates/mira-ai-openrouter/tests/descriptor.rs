#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! La fiche du fournisseur « openrouter » annonce sa connexion par OAuth PKCE et ne porte aucun secret.

use mira_ai_openrouter::OpenRouterAiProvider;
use mira_vocabulary::{AiProvider, AiProviderId, AuthScheme, WireFormat};

/// La fiche reprend les adresses documentées par OpenRouter
#[test]
fn descriptor_matches_the_documentation() {
    let d = OpenRouterAiProvider.descriptor();
    assert_eq!(d.id, AiProviderId("openrouter".to_string()));
    assert_eq!(d.display_name, "OpenRouter");
    assert_eq!(d.base_url, "https://openrouter.ai/api/v1");
    assert_eq!(d.auth, AuthScheme::Bearer);
    assert_eq!(d.format, WireFormat::OpenAiChat);
    assert_eq!(d.models_path, "/models");
    assert!(!d.base_url_editable);
}

/// La connexion par navigateur vise les deux adresses de la documentation, en HTTPS
#[test]
fn descriptor_declares_the_key_login() {
    let login = OpenRouterAiProvider.descriptor().key_login.unwrap();
    assert_eq!(login.authorize_url, "https://openrouter.ai/auth");
    assert_eq!(login.exchange_url, "https://openrouter.ai/api/v1/auth/keys");
}

/// Aucun champ ne porte de secret ni de nom de modèle
#[test]
fn descriptor_has_no_secret_and_no_model_name() {
    let d = OpenRouterAiProvider.descriptor();
    let text = format!("{d:?}").to_lowercase();
    for marker in ["claude-", "gpt-", "gemini-", "llama", "sk-or", "key="] {
        assert!(!text.contains(marker), "{text} contient {marker}");
    }
}
