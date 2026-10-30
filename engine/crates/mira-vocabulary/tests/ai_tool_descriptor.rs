#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Fiche d'un fournisseur d'IA : l'annonce des outils est le défaut d'une fiche plus ancienne.

use mira_vocabulary::{AiProviderDescriptor, AiProviderId, AiQuirks, AuthScheme, WireFormat};
use serde_json::Value;

/// Une fiche écrite avant les outils les annonce par défaut
#[test]
fn descriptor_without_supports_tools_defaults_to_true() {
    let mut fiche: Value = serde_json::to_value(sample_descriptor()).unwrap();
    fiche.as_object_mut().unwrap().remove("supports_tools");
    let read: AiProviderDescriptor = serde_json::from_value(fiche).unwrap();
    assert!(read.supports_tools);
}

/// Fiche d'exemple avec les outils annoncés.
fn sample_descriptor() -> AiProviderDescriptor {
    AiProviderDescriptor {
        id: AiProviderId("p".into()),
        display_name: "P".into(),
        base_url: "https://example.test".into(),
        auth: AuthScheme::Bearer,
        format: WireFormat::OpenAiChat,
        models_path: "/models".into(),
        model_exclusions: vec![],
        static_headers: vec![],
        quirks: AiQuirks::default(),
        base_url_editable: false,
        supports_tools: true,
    }
}
