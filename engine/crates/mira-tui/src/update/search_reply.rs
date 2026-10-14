//! Réponse du moteur à une recherche ; celle d'une requête qui n'est plus la courante est ignorée.

use mira_assembly::api::Response;
use mira_assembly::mira_vocabulary::MiraError;

use super::accounts_reply::{report, unexpected};
use crate::commands::Command;
use crate::state::AppState;

/// Remplace la liste par les résultats si la requête est toujours celle affichée.
pub fn on_search(
    state: &mut AppState,
    query: String,
    result: Result<Response, MiraError>,
) -> Vec<Command> {
    if state.search.query() != Some(query.as_str()) {
        return Vec::new();
    }
    match result {
        Ok(Response::Messages(page)) => state.messages.replace_items(page.items),
        Ok(_) => return unexpected(state),
        Err(error) => return report(state, error),
    }
    Vec::new()
}
