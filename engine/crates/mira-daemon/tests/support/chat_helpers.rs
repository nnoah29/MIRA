//! Aides des tests de chat : flux lent, statut d'une opération, attente de la coupure côté fournisseur.

use std::time::Duration;

use mira_testkit::ai::{Behavior, openai_stream};
use serde_json::{Value, json};

use super::TestClient;
use super::ai_daemon::AiDaemon;

/// Flux OpenAI de quarante morceaux espacés de 100 ms, assez lent pour être annulé en route.
pub fn slow_stream() -> Behavior {
    let texts: Vec<String> = (0..40).map(|index| format!("t{index} ")).collect();
    let refs: Vec<&str> = texts.iter().map(String::as_str).collect();
    match openai_stream(&refs, Some((3, 4))) {
        Behavior::Stream { chunks, .. } => Behavior::Stream {
            chunks,
            chunk_delay_ms: 100,
        },
        other => other,
    }
}

/// Statut JSON d'une opération, lu par le contrat.
pub async fn operation_status(client: &mut TestClient, operation: &str) -> Value {
    let request =
        json!({"domain": "operation", "request": {"op": "get_operation", "operation": operation}});
    client.call(request).await["data"]["data"]["status"].clone()
}

/// Attend que le faux serveur ait vu une connexion fermée par le client.
pub async fn wait_provider_closed(ai: &AiDaemon) {
    let limit = tokio::time::Instant::now() + Duration::from_secs(10);
    while ai.server.connections_closed_by_client() < 1 && tokio::time::Instant::now() < limit {
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    assert!(ai.server.connections_closed_by_client() >= 1);
}
