//! Rendu humain de la file d'actions et de la boîte d'envoi.

use mira_assembly::api::OutboxEntry;
use mira_assembly::mira_vocabulary::PendingAction;

use super::instant::format_instant;
use super::label::json_label;
use super::table::render_table;

/// Cellules d'une ligne d'action de la file.
fn action_row(action: &PendingAction) -> Vec<String> {
    vec![
        action.id.to_string(),
        json_label(&action.kind),
        json_label(&action.state),
        action.targets.len().to_string(),
        action.attempts.to_string(),
        action.last_error.clone().unwrap_or_else(|| "-".to_string()),
    ]
}

/// Cellules d'une ligne de la boîte d'envoi.
fn outbox_row(entry: &OutboxEntry) -> Vec<String> {
    vec![
        entry.action.to_string(),
        entry.draft.to_string(),
        json_label(&entry.state),
        format_instant(entry.scheduled_for),
        entry.subject.clone(),
        entry.last_error.clone().unwrap_or_else(|| "-".to_string()),
    ]
}

/// Tableau de la file : identifiant, nature, état, cibles, tentatives, dernière erreur.
pub fn queue_table(actions: &[PendingAction]) -> String {
    let rows: Vec<Vec<String>> = actions.iter().map(action_row).collect();
    render_table(
        &["ACTION", "NATURE", "ÉTAT", "CIBLES", "ESSAIS", "ERREUR"],
        &rows,
    )
}

/// Tableau de la boîte d'envoi : action, brouillon, état, instant prévu, objet, erreur.
pub fn outbox_table(entries: &[OutboxEntry]) -> String {
    let rows: Vec<Vec<String>> = entries.iter().map(outbox_row).collect();
    render_table(
        &["ACTION", "BROUILLON", "ÉTAT", "PRÉVU", "OBJET", "ERREUR"],
        &rows,
    )
}
