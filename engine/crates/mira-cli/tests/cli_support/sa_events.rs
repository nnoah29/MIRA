//! Lecture des événements rendus par `mira events` : types, positions, progression d'une étape.

use serde_json::Value;

/// Type de chaque événement (`kind.type`), dans l'ordre rendu.
pub fn event_types(events: &[Value]) -> Vec<String> {
    events
        .iter()
        .map(|event| event["kind"]["type"].as_str().unwrap().to_string())
        .collect()
}

/// Position du premier événement de ce type.
pub fn first_position(events: &[Value], kind: &str) -> Option<usize> {
    event_types(events).iter().position(|found| found == kind)
}

/// Nombre d'événements de ce type.
pub fn count_of(events: &[Value], kind: &str) -> usize {
    event_types(events)
        .iter()
        .filter(|found| *found == kind)
        .count()
}

/// Couples (fait, total) des événements de progression de cette étape, dans l'ordre.
pub fn progress_of(events: &[Value], step: &str) -> Vec<(u64, u64)> {
    let is_step =
        |event: &&Value| event["kind"]["type"] == "sync_progress" && event["kind"]["step"] == step;
    let pair = |event: &Value| {
        let kind = &event["kind"];
        (
            kind["done"].as_u64().unwrap(),
            kind["total"].as_u64().unwrap(),
        )
    };
    events.iter().filter(is_step).map(pair).collect()
}

/// Dit si les numéros d'événements croissent strictement.
pub fn seqs_strictly_increase(events: &[Value]) -> bool {
    let seqs: Vec<u64> = events.iter().map(|e| e["seq"].as_u64().unwrap()).collect();
    seqs.windows(2).all(|pair| pair[0] < pair[1])
}
