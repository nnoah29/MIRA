//! Réponse du moteur à la lecture des brouillons : seule celle attendue par la vue ouverte compte.

use mira_assembly::api::{Response, WriteResponse};
use mira_assembly::mira_vocabulary::MiraError;

use super::open::refresh_drafts;
use crate::commands::Command;
use crate::state::AppState;
use crate::update::accounts_reply::{report, unexpected};

/// Range les brouillons lus si c'est la lecture attendue ; un échec garde la liste affichée.
pub fn on_drafts(
    state: &mut AppState,
    token: u64,
    result: Result<Response, MiraError>,
) -> Vec<Command> {
    if !state.drafts.awaits_list(token) {
        return Vec::new();
    }
    let items = match result {
        Ok(Response::Write(WriteResponse::Drafts(drafts))) => Some(drafts),
        Ok(_) => {
            unexpected(state);
            None
        }
        Err(error) => {
            report(state, error);
            None
        }
    };
    if state.drafts.finish_list(items) {
        return refresh_drafts(state);
    }
    Vec::new()
}
