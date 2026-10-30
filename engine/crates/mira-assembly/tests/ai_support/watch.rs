//! Lecture du flux, annulation, fermeture côté serveur et journal des événements.

use std::time::Duration;

use mira_assembly::api::{ChatStream, EventRequest, OperationRequest, Request, Response};
use mira_assembly::mira_vocabulary::{ChatDelta, EventFilter, OperationId};
use mira_testkit::ai::{Behavior, FakeAiServer, openai_stream};
use tokio::time::{Instant, sleep, timeout};

use super::fixture::Fixture;

/// Lit tous les deltas jusqu'à la fin du flux (10 s au plus)
pub async fn collect(stream: &mut ChatStream) -> Vec<ChatDelta> {
    let mut deltas = Vec::new();
    let read_all = async {
        while let Some(delta) = stream.next().await {
            deltas.push(delta);
        }
    };
    timeout(Duration::from_secs(10), read_all).await.unwrap();
    deltas
}

/// Demande l'annulation de l'opération par le contrat, comme le ferait l'application
pub async fn cancel(fixture: &Fixture, operation: &OperationId) {
    let request = Request::Operation(OperationRequest::CancelOperation {
        operation: operation.clone(),
    });
    fixture.engine.execute(request).await.unwrap();
}

/// Attend qu'une connexion au moins ait été fermée par le client (10 s au plus)
pub async fn wait_closed(server: &FakeAiServer) -> bool {
    let deadline = Instant::now() + Duration::from_secs(10);
    while Instant::now() < deadline {
        if server.connections_closed_by_client() >= 1 {
            return true;
        }
        sleep(Duration::from_millis(20)).await;
    }
    false
}

/// Tous les événements du journal, avec leurs données, sous forme de texte
pub async fn events_text(fixture: &Fixture) -> String {
    let request = Request::Event(EventRequest::GetEvents {
        filter: EventFilter::default(),
        limit: 1000,
    });
    match fixture.engine.execute(request).await.unwrap() {
        Response::Events(events) => format!("{events:?}"),
        other => panic!("réponse inattendue : {other:?}"),
    }
}

/// Flux OpenAI de quarante morceaux espacés de 100 ms, assez lent pour être annulé en route
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
