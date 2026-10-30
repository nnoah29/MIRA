#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Liste des modèles : échecs sans effet sur le cache, fournisseur inconnu, clé absente.

mod ai_models_support;
mod support_api;

use ai_models_support::{ModelsFixture, TestProvider};
use mira_core::ai::models::fetch_models;
use mira_testkit::ai::Behavior;
use mira_vocabulary::{AiModel, AiModelList, Timestamp};

/// Liste gardée d'avance, reconnaissable
fn old_list() -> AiModelList {
    AiModelList {
        models: vec![AiModel {
            id: "old".to_string(),
            created_at: None,
        }],
        fetched_at: Timestamp::from_millis(42),
    }
}

/// Une réponse illisible est `server.ai_bad_response` et la liste gardée ne bouge pas
#[tokio::test]
async fn invalid_json_is_bad_response_and_cache_untouched() {
    let fixture = ModelsFixture::start(TestProvider::new("test-provider")).await;
    let connection = fixture.add_connection("test-provider", Some(fixture.server.base_url()));
    fixture.store_key("conn-1", "sk-test");
    fixture
        .db
        .write(|tx| tx.save_ai_models(&connection.id, &old_list()))
        .unwrap();
    fixture.server.queue(Behavior::Models {
        json: "pas du json".to_string(),
    });
    let error = fetch_models(&fixture.engine, &connection, true)
        .await
        .unwrap_err();
    assert_eq!(error.code, "server.ai_bad_response");
    assert_eq!(fixture.stored(&connection).models, Some(old_list()));
}

/// Un refus du service (clé rejetée) est rendu tel quel et la liste gardée ne bouge pas
#[tokio::test]
async fn rejected_key_leaves_cache_untouched() {
    let fixture = ModelsFixture::start(TestProvider::new("test-provider")).await;
    let connection = fixture.add_connection("test-provider", Some(fixture.server.base_url()));
    fixture.store_key("conn-1", "sk-test");
    fixture
        .db
        .write(|tx| tx.save_ai_models(&connection.id, &old_list()))
        .unwrap();
    let denied = Behavior::Status {
        code: 401,
        headers: Vec::new(),
        body: "{}".to_string(),
    };
    fixture.server.queue(denied);
    let error = fetch_models(&fixture.engine, &connection, true)
        .await
        .unwrap_err();
    assert_eq!(error.code, "authentication.ai_key_rejected");
    assert_eq!(fixture.stored(&connection).models, Some(old_list()));
}

/// Un modèle par défaut absent de la liste n'est pas une erreur ici : `ModelNotFound` n'arrive qu'à la génération
#[tokio::test]
async fn saved_default_model_missing_from_list_is_not_an_error_here() {
    let fixture = ModelsFixture::start(TestProvider::new("test-provider")).await;
    let connection = fixture.add_connection("test-provider", Some(fixture.server.base_url()));
    fixture.store_key("conn-1", "sk-test");
    let json = r#"{"data":[{"id":"autre","created":1}]}"#;
    fixture.server.queue(Behavior::Models {
        json: json.to_string(),
    });
    assert_eq!(connection.default_model.as_deref(), Some("model-a"));
    let list = fetch_models(&fixture.engine, &connection, false)
        .await
        .unwrap();
    assert_eq!(list.models[0].id, "autre");
}

/// Un fournisseur absent du registre donne `not_found.ai_provider`, sans appel réseau
#[tokio::test]
async fn unknown_provider_is_not_found() {
    let fixture = ModelsFixture::start(TestProvider::new("test-provider")).await;
    let connection = fixture.add_connection("other", Some(fixture.server.base_url()));
    let error = fetch_models(&fixture.engine, &connection, false)
        .await
        .unwrap_err();
    assert_eq!(error.code, "not_found.ai_provider");
    assert!(fixture.server.requests().is_empty());
}

/// Sans clé, une adresse hors boucle locale donne `incompatible_state.ai_key_missing`, sans appel réseau
#[tokio::test]
async fn missing_key_is_refused_outside_loopback() {
    let fixture = ModelsFixture::start(TestProvider::new("test-provider")).await;
    let remote = Some("https://api.provider.test/v1".to_string());
    let connection = fixture.add_connection("test-provider", remote);
    let error = fetch_models(&fixture.engine, &connection, false)
        .await
        .unwrap_err();
    assert_eq!(error.code, "incompatible_state.ai_key_missing");
    assert!(fixture.server.requests().is_empty());
}
