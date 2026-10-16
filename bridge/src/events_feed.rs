//! Flux des événements du moteur vers le client : jamais de rejeu du journal passé sans demande explicite.

use mira_assembly::api::{Engine, EventRequest, Request, Response};
use mira_assembly::mira_vocabulary::{EventFilter, EventSeq};
use serde_json::to_value;

use crate::session::current;

/// Numéro du dernier événement déjà enregistré (`None` si le journal est vide).
async fn latest_seq(engine: &Engine) -> Option<EventSeq> {
    let mut cursor: Option<EventSeq> = None;
    loop {
        let filter = EventFilter {
            after: cursor,
            ..EventFilter::default()
        };
        let request = Request::Event(EventRequest::GetEvents { filter, limit: 500 });
        let Ok(Response::Events(page)) = engine.execute(request).await else {
            return cursor;
        };
        let Some(last) = page.last() else {
            return cursor;
        };
        cursor = Some(last.seq);
        if page.len() < 500 {
            return cursor;
        }
    }
}

/// Envoie chaque événement (JSON) à `emit` ; sans `after`, seuls les événements futurs sont livrés (pas de rejeu du journal).
pub async fn pump_events(after: Option<u64>, mut emit: impl FnMut(String) -> bool) {
    let Some(engine) = current() else { return };
    let start = match after {
        Some(seq) => Some(EventSeq(seq)),
        None => latest_seq(&engine).await,
    };
    let filter = EventFilter {
        after: start,
        ..EventFilter::default()
    };
    let mut subscription = engine.subscribe(filter);
    drop(engine);
    while let Some(event) = subscription.next().await {
        let line = to_value(&event).map(|value| value.to_string());
        if line.is_ok_and(|line| !emit(line)) {
            return;
        }
    }
}
