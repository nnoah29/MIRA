//! Confirmation de fermeture d'un brouillon modifié : enregistrer, abandonner (ou continuer, qui ferme la fenêtre).

use super::close::{discard, save_and_close};
use super::intent::ComposeIntent;
use crate::commands::Command;
use crate::state::{AppState, Popup};

/// Applique le choix de l'utilisateur ; sans cette fenêtre, rien ne se passe.
pub fn route(state: &mut AppState, intent: ComposeIntent) -> Vec<Command> {
    if state.popup != Popup::ConfirmCloseDraft {
        return Vec::new();
    }
    match intent {
        ComposeIntent::SaveAndClose => save_and_close(state),
        ComposeIntent::Discard => discard(state),
        ComposeIntent::Reply
        | ComposeIntent::ReplyAll
        | ComposeIntent::Forward
        | ComposeIntent::New
        | ComposeIntent::Edit(_)
        | ComposeIntent::NextField
        | ComposeIntent::PrevField
        | ComposeIntent::Save
        | ComposeIntent::Close
        | ComposeIntent::Send => Vec::new(),
    }
}
