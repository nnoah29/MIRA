//! Confirmation de fermeture d'un brouillon modifié : enregistrer, abandonner en deux temps (`D` puis `D`), ou continuer.

use super::close::{discard, save_and_close};
use super::intent::ComposeIntent;
use crate::commands::Command;
use crate::state::{AppState, Popup};

/// Applique le choix de l'utilisateur selon la fenêtre ouverte ; sans fenêtre de fermeture, rien ne se passe.
#[rustfmt::skip]
pub fn route(state: &mut AppState, intent: ComposeIntent) -> Vec<Command> {
    let closing = state.popup == Popup::ConfirmCloseDraft;
    let discarding = state.popup == Popup::ConfirmDiscardDraft;
    match intent {
        ComposeIntent::SaveAndClose if closing => save_and_close(state),
        ComposeIntent::AskDiscard if closing => {
            state.popup = Popup::ConfirmDiscardDraft;
            Vec::new()
        }
        ComposeIntent::Discard if closing || discarding => discard(state),
        ComposeIntent::Reply | ComposeIntent::ReplyAll | ComposeIntent::Forward | ComposeIntent::New
        | ComposeIntent::Edit(_) | ComposeIntent::NextField | ComposeIntent::PrevField | ComposeIntent::Save
        | ComposeIntent::Close | ComposeIntent::Send | ComposeIntent::SaveAndClose | ComposeIntent::AskDiscard
        | ComposeIntent::Discard => Vec::new(),
    }
}
