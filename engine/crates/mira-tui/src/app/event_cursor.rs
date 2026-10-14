//! Dernier événement déjà journalisé à l'ouverture de la session, lu par pages sans y réagir (AG-41, PR-18).

use mira_assembly::api::{Engine, EventRequest, Request, Response};
use mira_assembly::mira_vocabulary::{EventFilter, EventSeq};

const PAGE: u32 = 1000;

/// Numéro du dernier événement du journal ; `None` si le journal est vide ou illisible.
pub async fn latest_event_seq(engine: &Engine) -> Option<EventSeq> {
    let mut last = None;
    loop {
        let page = events_after(engine, last).await;
        let Some(newest) = page.last() else {
            return last;
        };
        last = Some(newest.seq);
        if page.len() < PAGE as usize {
            return last;
        }
    }
}

/// Page d'au plus `PAGE` événements après ce numéro ; vide en cas d'erreur.
async fn events_after(
    engine: &Engine,
    after: Option<EventSeq>,
) -> Vec<mira_assembly::mira_vocabulary::Event> {
    let filter = EventFilter {
        after,
        ..EventFilter::default()
    };
    let request = Request::Event(EventRequest::GetEvents {
        filter,
        limit: PAGE,
    });
    match engine.execute(request).await {
        Ok(Response::Events(events)) => events,
        Ok(_) | Err(_) => Vec::new(),
    }
}
