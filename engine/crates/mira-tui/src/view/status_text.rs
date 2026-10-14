//! Texte de la barre d'état : indicateurs de synchro, de file d'actions et de connexion.

use crate::state::AppState;

/// Indicateurs actifs, dans l'ordre d'affichage.
pub fn indicators(state: &AppState) -> Vec<String> {
    let mut parts = Vec::new();
    if state.status.syncing {
        parts.push("Synchro en cours".to_string());
    }
    if state.status.offline {
        parts.push("Hors ligne".to_string());
    }
    match state.status.pending_actions {
        0 => {}
        1 => parts.push("1 action en file".to_string()),
        count => parts.push(format!("{count} actions en file")),
    }
    parts
}
