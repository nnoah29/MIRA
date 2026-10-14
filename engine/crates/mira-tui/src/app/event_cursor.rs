//! Dernier événement déjà journalisé à l'ouverture de la session, lu par pages sans y réagir (AG-41, PR-18).

use std::sync::Arc;

use mira_assembly::api::{Engine, EventRequest, Request, Response};
use mira_assembly::mira_vocabulary::{Event, EventFilter, EventSeq};

const PAGE: u32 = 1000;

/// Couture de test appelée après chaque page lue du journal, avec son rang.
#[derive(Clone)]
pub struct JournalProbe(pub Arc<dyn Fn(usize) + Send + Sync>);

impl std::fmt::Debug for JournalProbe {
    /// Affiche le type sans la fonction.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("JournalProbe")
    }
}

/// Numéro du dernier événement du journal ; `None` si le journal est vide ; une page illisible arrête le balayage là.
pub async fn latest_event_seq(engine: &Engine, probe: Option<&JournalProbe>) -> Option<EventSeq> {
    let mut last = None;
    for page_index in 0.. {
        let page = events_after(engine, last).await;
        if let Some(JournalProbe(probe)) = probe {
            probe(page_index);
        }
        let Some(newest) = page.last() else {
            return last;
        };
        last = Some(newest.seq);
        if page.len() < PAGE as usize {
            return last;
        }
    }
    last
}

/// Page d'au plus `PAGE` événements après ce numéro ; vide en cas d'erreur.
async fn events_after(engine: &Engine, after: Option<EventSeq>) -> Vec<Event> {
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
