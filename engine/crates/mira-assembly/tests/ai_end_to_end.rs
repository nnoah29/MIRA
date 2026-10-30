#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Assistant d'IA de bout en bout : vrai moteur assemblé, vrai fournisseur générique, faux serveur sur la boucle locale.

mod ai_support;

use ai_support::{
    KEY, ask_ai, cancel, chat, collect, connection_ready, consent_test_and_list_models,
    events_text, slow_stream, start, wait_closed,
};
use mira_assembly::api::AiRequest;
use mira_assembly::mira_vocabulary::{ChatDelta, ProfileId, StopReason};
use mira_assembly::{build_ai_registry, production_config_with_oauth};
use mira_testkit::ai::{Behavior, anthropic_stream, openai_stream};

const MODELS: &str = r#"{"data":[{"id":"model-a","created":100}]}"#;
const PROMPT: &str = "PROMPT-CONFIDENTIEL-17";
const REPLY: &str = "REPONSE-PRIVEE-23";

/// Le registre d'assemblage et celui de la configuration de production ont les six fournisseurs dans l'ordre
#[test]
fn registry_has_the_six_providers_in_order() {
    let expected = [
        "anthropic",
        "openai",
        "google",
        "mistral",
        "groq",
        "generic",
    ];
    let ids = |registry: mira_assembly::ai::AiProviderRegistry| -> Vec<String> {
        registry.list().into_iter().map(|item| item.id.0).collect()
    };
    assert_eq!(ids(build_ai_registry()), expected);
    let oauth = mira_assembly::auth::OAuthClientConfig::default();
    let config = production_config_with_oauth(ProfileId::from_raw("default"), oauth);
    assert_eq!(ids(config.ai_registry), expected);
}

/// Une connexion générique vers la boucle locale diffuse une réponse au format OpenAI, sans consentement
#[tokio::test]
async fn generic_connection_streams_openai_format_end_to_end() {
    let f = start().await;
    let connection = connection_ready(&f).await;
    f.server
        .queue(openai_stream(&["Bon", "jour"], Some((12, 5))));
    let mut stream = chat(&f, &connection.id, PROMPT).await;
    let deltas = collect(&mut stream).await;
    assert_eq!(
        deltas.last(),
        Some(&ChatDelta::Done {
            reason: StopReason::EndTurn
        })
    );
    let text: String = deltas
        .iter()
        .filter_map(|d| match d {
            ChatDelta::Text { text } => Some(text.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(text, "Bonjour");
    let sent = f.server.requests();
    assert_eq!(sent.len(), 1);
    assert_eq!(
        sent[0].header("authorization"),
        Some(format!("Bearer {KEY}").as_str())
    );
}

/// Le fournisseur générique est toujours au format OpenAI : un serveur au style Anthropic donne `ai_bad_response`
#[tokio::test]
async fn generic_connection_with_anthropic_style_server_is_not_supported() {
    let f = start().await;
    let connection = connection_ready(&f).await;
    f.server
        .queue(anthropic_stream(&["Bon", "jour"], Some((12, 5))));
    let mut stream = chat(&f, &connection.id, PROMPT).await;
    let deltas = collect(&mut stream).await;
    let failed = StopReason::Failed {
        code: "server.ai_bad_response".to_string(),
        retry_after_secs: None,
    };
    assert_eq!(deltas.last(), Some(&ChatDelta::Done { reason: failed }));
    assert!(
        deltas
            .iter()
            .all(|delta| !matches!(delta, ChatDelta::Text { .. }))
    );
}

/// Ajout, clé, consentement, essai, modèles puis discussion annulée : chaque étape passe par le contrat
#[tokio::test]
async fn full_flow_add_set_key_consent_test_models_chat_cancel() {
    let f = start().await;
    let connection = connection_ready(&f).await;
    assert!(connection.has_key);
    let id = connection.id.clone();
    f.server.queue(Behavior::Models {
        json: MODELS.to_string(),
    });
    f.server.queue(slow_stream());
    consent_test_and_list_models(&f, &id).await;
    assert_eq!(f.server.requests().len(), 1);
    let mut stream = chat(&f, &id, PROMPT).await;
    assert!(matches!(stream.next().await, Some(ChatDelta::Text { .. })));
    cancel(&f, &stream.operation).await;
    let rest = collect(&mut stream).await;
    let cancelled = ChatDelta::Done {
        reason: StopReason::Cancelled,
    };
    assert_eq!(rest.last(), Some(&cancelled));
    assert!(wait_closed(&f.server).await);
}

/// Ni la clé, ni la question, ni la réponse n'apparaissent dans le journal des événements
#[tokio::test]
async fn no_secret_or_prompt_in_event_journal_after_full_flow() {
    let f = start().await;
    let connection = connection_ready(&f).await;
    ask_ai(
        &f,
        AiRequest::AcknowledgeAiConsent {
            connection: connection.id.clone(),
        },
    )
    .await;
    f.server.queue(openai_stream(&[REPLY], Some((3, 4))));
    let mut stream = chat(&f, &connection.id, PROMPT).await;
    collect(&mut stream).await;
    let journal = events_text(&f).await;
    assert!(journal.contains("AiChatStarted") && journal.contains("AiChatFinished"));
    for secret in [KEY, PROMPT, REPLY] {
        assert!(
            !journal.contains(secret),
            "fuite de {secret} dans {journal}"
        );
    }
}

/// Les fournisseurs à adresse fixe pointent tous vers du HTTPS ; seul le service compatible a une adresse libre
#[test]
fn fixed_provider_descriptors_have_https_base_urls() {
    let descriptors = build_ai_registry().list();
    assert_eq!(descriptors.len(), 6);
    for descriptor in &descriptors {
        if descriptor.base_url_editable {
            assert_eq!(descriptor.id.0, "generic");
            assert!(descriptor.base_url.is_empty());
        } else {
            assert!(
                descriptor.base_url.starts_with("https://"),
                "{}",
                descriptor.id.0
            );
            assert!(!descriptor.base_url.ends_with('/'));
        }
    }
}
