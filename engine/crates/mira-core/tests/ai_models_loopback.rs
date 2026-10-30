#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Liste des modèles : sur la boucle locale la clé est facultative, ailleurs elle est exigée.

mod ai_models_support;
mod support_api;

use ai_models_support::{ModelsFixture, TestProvider};
use mira_core::ai::models::fetch_models;
use mira_testkit::ai::Behavior;
use mira_vocabulary::{AuthScheme, WireFormat};

const ONE_MODEL: &str = r#"{"data":[{"id":"local-model"}]}"#;

/// Sans clé rangée, un serveur local est interrogé quand même, avec une clé vide
#[tokio::test]
async fn loopback_without_key_still_fetches() {
    let fixture = ModelsFixture::start(TestProvider::new("test-provider")).await;
    let connection = fixture.add_connection("test-provider", Some(fixture.server.base_url()));
    fixture.server.queue(Behavior::Models {
        json: ONE_MODEL.to_string(),
    });
    let list = fetch_models(&fixture.engine, &connection, false)
        .await
        .unwrap();
    assert_eq!(list.models[0].id, "local-model");
    let requests = fixture.server.requests();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].header("authorization"), Some("Bearer"));
}

/// Sans adresse propre à la connexion, celle du descripteur sert
#[tokio::test]
async fn descriptor_base_url_is_used_when_connection_has_none() {
    let fixture = ModelsFixture::start(TestProvider::new("test-provider")).await;
    let connection = fixture.add_connection("test-provider", None);
    fixture.store_key("conn-1", "sk-test");
    fixture.server.queue(Behavior::Models {
        json: ONE_MODEL.to_string(),
    });
    fetch_models(&fixture.engine, &connection, false)
        .await
        .unwrap();
    assert_eq!(fixture.server.requests().len(), 1);
}

/// Le format Messages envoie la clé en `x-api-key` et décode ses dates RFC 3339
#[tokio::test]
async fn anthropic_style_provider_uses_x_api_key() {
    let mut provider = TestProvider::new("test-provider");
    provider.auth = AuthScheme::XApiKey;
    provider.format = WireFormat::AnthropicMessages;
    let fixture = ModelsFixture::start(provider).await;
    let connection = fixture.add_connection("test-provider", Some(fixture.server.base_url()));
    fixture.store_key("conn-1", "sk-test-SECRET");
    let json = r#"{"data":[{"id":"m1","created_at":"2025-02-19T00:00:00Z"}]}"#;
    fixture.server.queue(Behavior::Models {
        json: json.to_string(),
    });
    let list = fetch_models(&fixture.engine, &connection, false)
        .await
        .unwrap();
    assert_eq!(list.models[0].created_at, Some(1_739_923_200));
    let requests = fixture.server.requests();
    assert_eq!(requests[0].header("x-api-key"), Some("sk-test-SECRET"));
}
