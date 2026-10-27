//! Lecture du texte complet des erreurs et des événements, pour vérifier qu'aucun secret n'y figure.

use mira_core::api::{Engine, EventRequest, Request, Response};
use mira_vocabulary::{EventFilter, MiraError};

/// Erreur entière (code, message, contexte) d'une requête qui doit échouer
pub async fn failure_text(engine: &Engine, request: Request) -> String {
    let error: MiraError = engine.execute(request).await.unwrap_err();
    format!("{error:?}")
}

/// Tous les événements du moteur avec leurs données, sous forme de texte
pub async fn events_text(engine: &Engine) -> String {
    let request = Request::Event(EventRequest::GetEvents {
        filter: EventFilter::default(),
        limit: 1000,
    });
    match engine.execute(request).await.unwrap() {
        Response::Events(events) => format!("{events:?}"),
        other => panic!("réponse inattendue : {other:?}"),
    }
}
