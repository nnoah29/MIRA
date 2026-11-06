//! Un consommateur qui se ferme en cours de route annule l'opération et la connexion au fournisseur.

use std::time::Duration;

use mira_testkit::ai::{Behavior, openai_stream};
use serde_json::Value;

use super::chat_fixture::chat_json;
use super::chat_start::{operation_status, start};
use super::driver::Driver;

/// Flux OpenAI de quarante morceaux espacés de 100 ms, assez lent pour être annulé en route.
fn slow_stream() -> Behavior {
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

/// Quand le consommateur se ferme, l'opération est annulée et la connexion au fournisseur fermée.
pub async fn a_closed_consumer_cancels_the_operation<D: Driver>() {
    let f = start::<D>().await;
    f.server.queue(slow_stream());
    let mut lines: Vec<Value> = Vec::new();
    let request = chat_json(Some(&f.connection), "Salut");
    f.driver
        .pump_chat(&request, |line| {
            lines.push(serde_json::from_str(&line).unwrap());
            lines.len() < 2
        })
        .await;
    assert_eq!(lines.len(), 2, "{lines:?}");
    let operation = lines[0]["operation"].as_str().unwrap();
    let limit = tokio::time::Instant::now() + Duration::from_secs(10);
    while f.server.connections_closed_by_client() < 1 && tokio::time::Instant::now() < limit {
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    assert!(f.server.connections_closed_by_client() >= 1);
    assert_eq!(
        operation_status(&f.driver, operation).await["type"],
        "cancelled"
    );
    assert!(f.driver.close().await.contains("\"ok\":true"));
}
