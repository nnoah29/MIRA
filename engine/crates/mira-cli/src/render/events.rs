//! Rendu humain des événements.

use mira_assembly::mira_vocabulary::Event;

use super::label::json_label;
use super::table::render_table;

/// Cellules d'une ligne d'événement.
fn event_row(event: &Event) -> Vec<String> {
    vec![
        event.seq.0.to_string(),
        event.at.as_millis().to_string(),
        json_label(&event.kind),
    ]
}

/// Tableau des événements : numéro, instant (ms Unix), type.
pub fn events_table(events: &[Event]) -> String {
    let rows: Vec<Vec<String>> = events.iter().map(event_row).collect();
    render_table(&["SEQ", "INSTANT_MS", "TYPE"], &rows)
}
