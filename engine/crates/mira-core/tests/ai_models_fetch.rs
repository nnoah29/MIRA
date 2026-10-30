#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Liste des modèles d'une connexion : appel du serveur, cache, rafraîchissement.

mod ai_models_support;
mod support_api;

use ai_models_support::{ModelsFixture, TestProvider};
use mira_core::ai::models::fetch_models;
use mira_testkit::ai::Behavior;
use mira_vocabulary::Timestamp;

const TWO_MODELS: &str =
    r#"{"data":[{"id":"model-a","created":100},{"id":"model-b","created":200}]}"#;

/// Fixture avec une connexion pointant sur le faux serveur et une clé rangée
async fn started(provider: TestProvider) -> (ModelsFixture, mira_vocabulary::AiConnection) {
    let fixture = ModelsFixture::start(provider).await;
    let connection = fixture.add_connection("test-provider", Some(fixture.server.base_url()));
    fixture.store_key("conn-1", "sk-test-SECRET");
    (fixture, connection)
}

/// Le serveur est interrogé avec la clé, la liste est triée, rendue et gardée en base avec la date du moteur
#[tokio::test]
async fn fetch_models_calls_the_server_and_caches() {
    let (fixture, connection) = started(TestProvider::new("test-provider")).await;
    fixture.server.queue(Behavior::Models {
        json: TWO_MODELS.to_string(),
    });
    let list = fetch_models(&fixture.engine, &connection, false)
        .await
        .unwrap();
    let ids: Vec<&str> = list.models.iter().map(|item| item.id.as_str()).collect();
    assert_eq!(ids, ["model-b", "model-a"]);
    assert_eq!(list.fetched_at, Timestamp::from_millis(1_700_000_000_000));
    let requests = fixture.server.requests();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].path, "/models");
    assert_eq!(
        requests[0].header("authorization"),
        Some("Bearer sk-test-SECRET")
    );
    assert_eq!(fixture.stored(&connection).models, Some(list));
}

/// Avec une liste déjà gardée et sans rafraîchissement, le serveur n'est pas interrogé
#[tokio::test]
async fn fetch_models_uses_cache_without_refresh() {
    let (fixture, mut connection) = started(TestProvider::new("test-provider")).await;
    fixture.server.queue(Behavior::Models {
        json: TWO_MODELS.to_string(),
    });
    let first = fetch_models(&fixture.engine, &connection, false)
        .await
        .unwrap();
    connection.models = Some(first.clone());
    let again = fetch_models(&fixture.engine, &connection, false)
        .await
        .unwrap();
    assert_eq!(again, first);
    assert_eq!(fixture.server.requests().len(), 1);
}

/// Un rafraîchissement interroge de nouveau le serveur et remplace la liste gardée
#[tokio::test]
async fn fetch_models_refresh_replaces_cache() {
    let (fixture, mut connection) = started(TestProvider::new("test-provider")).await;
    let renewed = r#"{"data":[{"id":"model-c","created":300}]}"#;
    fixture.server.queue(Behavior::Models {
        json: TWO_MODELS.to_string(),
    });
    fixture.server.queue(Behavior::Models {
        json: renewed.to_string(),
    });
    connection.models = Some(
        fetch_models(&fixture.engine, &connection, false)
            .await
            .unwrap(),
    );
    let list = fetch_models(&fixture.engine, &connection, true)
        .await
        .unwrap();
    assert_eq!(list.models.len(), 1);
    assert_eq!(fixture.server.requests().len(), 2);
    assert_eq!(fixture.stored(&connection).models, Some(list));
}

/// Une liste vide du serveur est valide et gardée telle quelle ; elle sert ensuite de cache
#[tokio::test]
async fn empty_server_list_is_ok_and_cached_empty() {
    let (fixture, mut connection) = started(TestProvider::new("test-provider")).await;
    fixture.server.queue(Behavior::Models {
        json: r#"{"data":[]}"#.to_string(),
    });
    let list = fetch_models(&fixture.engine, &connection, false)
        .await
        .unwrap();
    assert!(list.models.is_empty());
    assert_eq!(fixture.stored(&connection).models, Some(list));
    connection.models = fixture.stored(&connection).models;
    fetch_models(&fixture.engine, &connection, false)
        .await
        .unwrap();
    assert_eq!(fixture.server.requests().len(), 1);
}

/// Les exclusions du fournisseur retirent les modèles qui ne discutent pas, avant le tri
#[tokio::test]
async fn fetch_models_applies_provider_exclusions() {
    let mut provider = TestProvider::new("test-provider");
    provider.exclusions = vec!["embed".to_string()];
    let (fixture, connection) = started(provider).await;
    let json = r#"{"data":[{"id":"Text-Embed-1","created":900},{"id":"model-a","created":1}]}"#;
    fixture.server.queue(Behavior::Models {
        json: json.to_string(),
    });
    let list = fetch_models(&fixture.engine, &connection, false)
        .await
        .unwrap();
    assert_eq!(list.models.len(), 1);
    assert_eq!(list.models[0].id, "model-a");
}
