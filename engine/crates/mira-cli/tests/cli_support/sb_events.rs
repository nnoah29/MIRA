//! Lecture des événements et des actions rendus en JSON par le CLI.

use serde_json::Value;

/// Contenus (`kind`) des événements de ce type, dans l'ordre des numéros.
pub fn events_of_type(events: &[Value], kind: &str) -> Vec<Value> {
    events
        .iter()
        .map(|event| event["kind"].clone())
        .filter(|found| found["type"] == kind)
        .collect()
}

/// Numéro du premier événement de ce type, s'il y en a.
pub fn first_seq_of_type(events: &[Value], kind: &str) -> Option<u64> {
    let found = events.iter().find(|event| event["kind"]["type"] == kind);
    found.and_then(|event| event["seq"].as_u64())
}

/// Action de la file portant cet identifiant.
pub fn action_by_id(actions: &[Value], id: &str) -> Value {
    let found = actions.iter().find(|action| action["id"] == id);
    found.unwrap().clone()
}
