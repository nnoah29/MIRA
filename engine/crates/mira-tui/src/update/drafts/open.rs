//! Ouverture, fermeture et relecture de la vue « Brouillons » du compte sélectionné.

use mira_assembly::api::{ComposeRequest, Request};
use mira_assembly::mira_vocabulary::AccountId;

use crate::commands::{Command, Tag};
use crate::state::{AppState, Popup};
use crate::update::account_switch::selected_id;

/// Ouvre la vue sur le compte sélectionné et lit ses brouillons.
pub fn open_drafts(state: &mut AppState) -> Vec<Command> {
    let Some(account) = selected_id(state) else {
        state.status.message = Some("Aucun compte sélectionné".to_string());
        return Vec::new();
    };
    state.drafts.open(account);
    refresh_drafts(state)
}

/// Referme la vue et sa confirmation de suppression ; les réponses encore attendues seront ignorées.
pub fn close_drafts(state: &mut AppState) -> Vec<Command> {
    if state.drafts.is_opening() {
        state.status.message = None;
    }
    if matches!(state.popup, Popup::ConfirmDeleteDraft { .. }) {
        state.popup = Popup::Closed;
    }
    state.drafts.close();
    Vec::new()
}

/// Relit la vue ouverte ; pendant une lecture, une seule de plus est notée pour la fin.
pub fn refresh_drafts(state: &mut AppState) -> Vec<Command> {
    let Some(token) = state.drafts.start_list() else {
        return Vec::new();
    };
    match state.drafts.account() {
        Some(account) => vec![list_command(account, token)],
        None => Vec::new(),
    }
}

/// Lecture des brouillons du compte, suivie par ce jeton.
fn list_command(account: &AccountId, token: u64) -> Command {
    Command::Execute {
        tag: Tag::Drafts(token),
        request: Request::Compose(ComposeRequest::ListDrafts {
            account: account.clone(),
        }),
    }
}
