//! Session complète du domaine « Ai » dont on garde tout ce qui sort, pour chercher une fuite de clé.

use mira_core::api::{AiRequest, Engine, SecretInput};
use mira_testkit::ai::Behavior;
use mira_vocabulary::AiConnectionId;

use super::calls::{call, list};
use super::operations::{consent, models, set_default, set_key, update};

/// Rejoue toutes les opérations avec cette clé et rend les réponses, erreurs et requêtes sérialisées
pub async fn run_session(engine: &Engine, id: &AiConnectionId, url: &str, key: &str) -> String {
    let mut seen = vec![serde_json::to_string(
        &set_key(engine, id, key).await.unwrap(),
    )];
    seen.push(serde_json::to_string(&consent(engine, id).await));
    seen.push(serde_json::to_string(&set_default(engine, id).await));
    seen.push(serde_json::to_string(
        &models(engine, id, true).await.unwrap(),
    ));
    let updated = update(engine, id, "B", Some(url), Some("m")).await.unwrap();
    seen.push(serde_json::to_string(&updated));
    seen.push(serde_json::to_string(&list(engine).await));
    let refused = set_key(engine, id, &format!("{key} x")).await.unwrap_err();
    let request = AiRequest::SetAiApiKey {
        connection: id.clone(),
        key: SecretInput::new(key),
    };
    let removed = call(
        engine,
        AiRequest::RemoveAiConnection {
            connection: id.clone(),
        },
    )
    .await;
    format!("{seen:?}{refused}{request:?}{removed:?}")
}

/// Met en file une liste d'un seul modèle pour la session
pub fn queue_one_model(server: &mira_testkit::ai::FakeAiServer) {
    server.queue(Behavior::Models {
        json: r#"{"data":[{"id":"m"}]}"#.to_string(),
    });
}
