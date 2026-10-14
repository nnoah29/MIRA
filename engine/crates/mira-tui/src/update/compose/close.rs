//! Fermeture de l'éditeur : confirmation si des modifications ne sont pas enregistrées, suppression d'un brouillon intact.

use super::save::save;
use crate::commands::{Command, Tag};
use crate::state::{AppState, ComposeSlot, Popup};
use mira_assembly::api::{ComposeRequest, Request};

/// Échap : ferme tout de suite, ou demande confirmation s'il reste des modifications non enregistrées.
pub fn request_close(state: &mut AppState) -> Vec<Command> {
    let unsaved = state.compose.open().is_some_and(|c| c.has_unsaved());
    if unsaved {
        state.popup = Popup::ConfirmCloseDraft;
        return Vec::new();
    }
    close_now(state)
}

/// Ferme l'éditeur ; un brouillon que l'utilisateur n'a jamais enregistré est supprimé du moteur.
pub fn close_now(state: &mut AppState) -> Vec<Command> {
    let ComposeSlot::Open(compose) = std::mem::take(&mut state.compose) else {
        return Vec::new();
    };
    if compose.saved_once() {
        return Vec::new();
    }
    let draft = compose.draft().clone();
    vec![Command::Execute {
        tag: Tag::DraftDeleted(draft.clone()),
        request: Request::Compose(ComposeRequest::DeleteDraft { draft }),
    }]
}

/// Abandonne les modifications non enregistrées et ferme.
pub fn discard(state: &mut AppState) -> Vec<Command> {
    state.popup = Popup::Closed;
    close_now(state)
}

/// Enregistre puis ferme dès que le moteur a confirmé ; l'éditeur reste ouvert si rien ne peut partir.
pub fn save_and_close(state: &mut AppState) -> Vec<Command> {
    state.popup = Popup::Closed;
    if state.compose.open().is_some_and(|c| c.is_saving()) {
        mark_close_after_save(state, true);
        return Vec::new();
    }
    let commands = save(state);
    mark_close_after_save(state, !commands.is_empty());
    commands
}

/// Après un enregistrement réussi : ferme si la fermeture était demandée et que tout est enregistré, sinon renvoie le reste.
pub fn continue_close(state: &mut AppState) -> Vec<Command> {
    let wanted = state
        .compose
        .open_mut()
        .is_some_and(|c| c.take_close_after_save());
    if !wanted {
        return Vec::new();
    }
    if state.compose.open().is_some_and(|c| !c.has_unsaved()) {
        return close_now(state);
    }
    let commands = save(state);
    mark_close_after_save(state, !commands.is_empty());
    commands
}

/// Note si la fermeture doit suivre l'enregistrement en cours.
fn mark_close_after_save(state: &mut AppState, close: bool) {
    if let Some(compose) = state.compose.open_mut() {
        compose.set_close_after_save(close);
    }
}
