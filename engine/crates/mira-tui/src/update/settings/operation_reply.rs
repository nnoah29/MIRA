//! Réponse à une modification : annoncée dans la barre d'état, succès comme échec, puis lectures à refaire.

use mira_assembly::api::Response;
use mira_assembly::mira_vocabulary::MiraError;

use super::follow_up::follow_up;
use crate::commands::Command;
use crate::state::AppState;
use crate::state::settings::{Operation, SettingsTab};
use crate::update::accounts_reply::report;
use crate::view::safe_text::display_safe;

/// Annonce l'issue de la modification attendue sous ce jeton ; une réponse périmée est ignorée.
pub fn on_operation(
    state: &mut AppState,
    token: u64,
    result: Result<Response, MiraError>,
) -> Vec<Command> {
    let Some(operation) = state.settings.take_operation(token) else {
        return Vec::new();
    };
    match result {
        Ok(response) => {
            announce(state, operation, &response);
            follow_up(state, operation.follow_up())
        }
        Err(error) => report(state, error),
    }
}

/// Texte de l'issue : verdict du contrôle, fichier écrit, ou message propre à l'opération.
fn describe(operation: Operation, response: &Response) -> String {
    match response {
        Response::Integrity(verdict) if verdict.ok => "Base intègre".to_string(),
        Response::Integrity(_) => "Base endommagée".to_string(),
        Response::Exported(done) => format!(
            "Export écrit : {} ({} octets)",
            display_safe(&done.path),
            done.bytes
        ),
        _ => operation.done_message().to_string(),
    }
}

/// Montre l'issue dans la barre d'état et, sur la page Maintenance, en compte rendu.
fn announce(state: &mut AppState, operation: Operation, response: &Response) {
    let text = describe(operation, response);
    if state.settings.is_open() && state.settings.tab == SettingsTab::Maintenance {
        state.settings.maintenance.report = Some(text.clone());
    }
    state.status.message = Some(text);
}
