#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! La fiche du fournisseur « generic » reprend la table du plan et ne porte ni secret ni nom de modèle.

use mira_ai_generic::GenericAiProvider;
use mira_vocabulary::{AiProvider, AiProviderId, AuthScheme, MaxTokensField, WireFormat};

/// La fiche reprend exactement les valeurs documentées du fournisseur
#[test]
fn descriptor_matches_the_table() {
    let d = GenericAiProvider.descriptor();
    assert_eq!(d.id, AiProviderId("generic".to_string()));
    assert_eq!(d.display_name, "Compatible OpenAI");
    assert_eq!(d.base_url, "");
    assert_eq!(d.auth, AuthScheme::Bearer);
    assert_eq!(d.format, WireFormat::OpenAiChat);
    assert_eq!(d.models_path, "/models");
    assert_eq!(d.quirks.max_tokens_field, MaxTokensField::MaxTokens);
    assert!(d.base_url_editable);
}

/// Les exclusions de modèles et les en-têtes fixes reprennent la table
#[test]
fn descriptor_exclusions_and_headers_match_the_table() {
    let d = GenericAiProvider.descriptor();
    assert_eq!(d.model_exclusions, Vec::<String>::new());
    assert!(d.static_headers.is_empty());
}

/// Aucun champ ne porte de secret ni de nom de modèle
#[test]
fn descriptor_has_no_secret_and_no_model_name() {
    let d = GenericAiProvider.descriptor();
    let mut texts = vec![
        d.display_name.clone(),
        d.base_url.clone(),
        d.models_path.clone(),
    ];
    for (name, value) in &d.static_headers {
        texts.push(name.clone());
        texts.push(value.clone());
    }
    for text in texts.iter().map(|t| t.to_lowercase()) {
        for marker in [
            "claude-", "gpt-", "gemini-", "mistral-", "llama", "sk-", "key=",
        ] {
            assert!(!text.contains(marker), "{text} contient {marker}");
        }
    }
    assert!(d.model_exclusions.iter().all(|w| *w == w.to_lowercase()));
}

/// Le fournisseur s'utilise comme objet de trait partageable entre fils
#[test]
fn provider_is_usable_as_a_shared_trait_object() {
    let boxed: Box<dyn AiProvider> = Box::new(GenericAiProvider);
    assert_eq!(boxed.descriptor().id, AiProviderId("generic".to_string()));
}
