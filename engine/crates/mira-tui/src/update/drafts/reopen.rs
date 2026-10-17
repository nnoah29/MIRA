//! Réouverture d'un brouillon enregistré : relu au moteur, ouvert seulement s'il est encore en édition.

use mira_assembly::api::{ComposeRequest, Request, Response, WriteResponse};
use mira_assembly::mira_vocabulary::{Draft, MiraError};

use super::target::{not_editable_reason, say, selected_draft};
use crate::commands::{Command, Tag};
use crate::state::compose::ComposeState;
use crate::state::{AppState, ComposeSlot, Popup};
use crate::update::accounts_reply::{report, unexpected};

/// Entrée : relit le brouillon sélectionné pour le rouvrir ; un brouillon planifié ou en envoi reste dans la vue.
pub fn reopen_selected(state: &mut AppState) -> Vec<Command> {
    if state.drafts.is_opening() {
        return say(state, "Ouverture du brouillon en cours");
    }
    let Some(draft) = selected_draft(state) else {
        return Vec::new();
    };
    if let Some(reason) = not_editable_reason(&draft) {
        return say(state, reason);
    }
    let token = state.drafts.begin_open();
    state.status.message = Some("Ouverture du brouillon…".to_string());
    vec![Command::Execute {
        tag: Tag::DraftLoaded(token),
        request: Request::Compose(ComposeRequest::GetDraft { draft: draft.id }),
    }]
}

/// Réponse à la relecture : seule celle attendue par la vue ouverte compte, et jamais par-dessus un autre brouillon.
pub fn on_draft_loaded(
    state: &mut AppState,
    token: u64,
    result: Result<Response, MiraError>,
) -> Vec<Command> {
    if !state.drafts.take_open(token) || !state.compose.is_closed() {
        return Vec::new();
    }
    match result {
        Ok(Response::Write(WriteResponse::Draft(draft))) => open_in_editor(state, &draft),
        Ok(_) => unexpected(state),
        Err(error) => report(state, error),
    }
}

/// Ouvre l'éditeur sur ce brouillon (déjà enregistré) et ferme la vue ; s'il n'est plus en édition, la vue reste.
fn open_in_editor(state: &mut AppState, draft: &Draft) -> Vec<Command> {
    if let Some(reason) = not_editable_reason(draft) {
        return say(state, reason);
    }
    state.compose = ComposeSlot::Open(Box::new(ComposeState::from_saved_draft(draft)));
    state.drafts.close();
    state.popup = Popup::Closed;
    state.search.close_input();
    state.status.message = None;
    Vec::new()
}
