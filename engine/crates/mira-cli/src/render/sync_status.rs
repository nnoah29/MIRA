//! Rendu humain de l'état de synchronisation : un tableau, une ligne par compte.

use mira_assembly::api::{AccountSyncStatus, SyncResponse};

use super::instant::format_optional_instant;
use super::label::json_label;
use super::table::render_table;

/// Texte de la colonne « boîtes en cours » : leurs identifiants, ou un tiret.
fn mailboxes_cell(status: &AccountSyncStatus) -> String {
    if status.mailboxes_in_progress.is_empty() {
        return "-".to_string();
    }
    let ids: Vec<String> = status
        .mailboxes_in_progress
        .iter()
        .map(ToString::to_string)
        .collect();
    ids.join(",")
}

/// Cellules d'une ligne de compte.
fn status_row(status: &AccountSyncStatus) -> Vec<String> {
    let activity = if status.syncing {
        "en cours"
    } else {
        "au repos"
    };
    vec![
        status.account.to_string(),
        json_label(&status.state),
        activity.to_string(),
        mailboxes_cell(status),
        format_optional_instant(status.last_synced_at),
        status.pending_actions.to_string(),
        status.last_error.clone().unwrap_or_else(|| "-".to_string()),
    ]
}

/// Tableau des états : compte, état, synchro, boîtes en cours, dernier succès, actions en file, erreur.
pub fn sync_text(response: &SyncResponse) -> String {
    let statuses = match response {
        SyncResponse::Status(list) => list.as_slice(),
        SyncResponse::Paused(one) => std::slice::from_ref(one),
    };
    let rows: Vec<Vec<String>> = statuses.iter().map(status_row).collect();
    let headers = [
        "COMPTE",
        "ÉTAT",
        "SYNCHRO",
        "BOÎTES",
        "DERNIER SUCCÈS",
        "ACTIONS",
        "ERREUR",
    ];
    render_table(&headers, &rows)
}
