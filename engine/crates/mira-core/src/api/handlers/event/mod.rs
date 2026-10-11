//! Domaine « événements » : lecture par numéro pour les clients sans abonnement.

use mira_vocabulary::{EventFilter, MiraError};

use crate::api::contract::{EventRequest, Response};
use crate::api::engine::Engine;

const MAX_EVENTS_PER_PAGE: u32 = 1000;

/// Route une requête du domaine événements
pub(crate) async fn route(engine: &Engine, request: EventRequest) -> Result<Response, MiraError> {
    match request {
        EventRequest::GetEvents { filter, limit } => handle_get_events(engine, &filter, limit),
    }
}

/// Rend les événements après le numéro du filtre, bornés en nombre
fn handle_get_events(
    engine: &Engine,
    filter: &EventFilter,
    limit: u32,
) -> Result<Response, MiraError> {
    let bounded = limit.clamp(1, MAX_EVENTS_PER_PAGE);
    Ok(Response::Events(engine.events.catch_up(filter, bounded)?))
}
