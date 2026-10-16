//! Logique du pont, sans dépendance à flutter_rust_bridge : ouvrir, exécuter, observer, fermer.

use std::sync::Arc;
use std::time::Duration;

use mira_assembly::api::{Engine, EngineConfig, Request};
use mira_assembly::mira_vocabulary::{CONTRACT_VERSION, EventFilter, EventSeq};
use serde_json::{Value, to_value};

use crate::envelope::{failure, internal_failure, success};
use crate::session::{current, install, take};

const SHUTDOWN_DEADLINE: Duration = Duration::from_secs(5);

/// Ouvre le moteur sur cette configuration et rend l'enveloppe JSON du résultat.
pub fn open(config: EngineConfig) -> String {
    match Engine::open(config, CONTRACT_VERSION) {
        Ok(engine) => replace_engine(Arc::new(engine)),
        Err(error) => failure(&error),
    }
}

/// Range le nouveau moteur ; l'ancien, s'il existe, est fermé de force.
fn replace_engine(engine: Arc<Engine>) -> String {
    if let Some(previous) = install(engine) {
        drop(previous);
    }
    success(Value::Null)
}

/// Exécute une requête du contrat donnée en JSON et rend l'enveloppe JSON de la réponse.
pub async fn execute(request_json: &str) -> String {
    let Some(engine) = current() else {
        return internal_failure("moteur non ouvert");
    };
    let request: Request = match serde_json::from_str(request_json) {
        Ok(request) => request,
        Err(_) => return internal_failure("requête JSON illisible"),
    };
    match engine.execute(request).await {
        Ok(response) => to_value(&response).map_or_else(|_| internal_failure("réponse"), success),
        Err(error) => failure(&error),
    }
}

/// Envoie chaque événement (JSON) à `emit` depuis le numéro `after` ; s'arrête quand `emit` rend `false`.
pub async fn pump_events(after: Option<u64>, mut emit: impl FnMut(String) -> bool) {
    let Some(engine) = current() else { return };
    let filter = EventFilter {
        after: after.map(EventSeq),
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

/// Ferme le moteur en laissant quelques secondes aux synchronisations ; rend l'enveloppe du résultat.
pub async fn close() -> String {
    let Some(engine) = take() else {
        return success(Value::Null);
    };
    match Arc::try_unwrap(engine) {
        Ok(owned) => finish(owned).await,
        Err(_) => internal_failure("moteur encore utilisé"),
    }
}

/// Termine proprement un moteur dont on est l'unique propriétaire.
async fn finish(engine: Engine) -> String {
    match engine.shutdown_within(SHUTDOWN_DEADLINE).await {
        Ok(_) => success(Value::Null),
        Err(error) => failure(&error),
    }
}
