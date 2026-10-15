//! Suppression d'un brouillon : `D` ouvre la confirmation, seul un second `D` immédiat supprime.

use mira_assembly::api::{ComposeRequest, Request, Response};
use mira_assembly::mira_vocabulary::MiraError;

use super::intent::DraftsIntent;
use super::open::refresh_drafts;
use super::target::{say, selected_draft, subject_label};
use crate::commands::{Command, Tag};
use crate::state::{AppState, Popup};
use crate::update::accounts_reply::report;

/// Premier `D` : ouvre la confirmation sur le brouillon sélectionné (lui seul, par son identifiant).
pub fn ask_delete(state: &mut AppState) -> Vec<Command> {
    if state.drafts.is_opening() {
        return say(state, "Ouverture du brouillon en cours");
    }
    let Some(draft) = selected_draft(state) else {
        return Vec::new();
    };
    state.popup = Popup::ConfirmDeleteDraft {
        draft: draft.id,
        subject: draft.subject,
    };
    Vec::new()
}

/// Intention reçue sous une fenêtre : seul le second `D` sous la confirmation de suppression agit.
pub fn route_popup(state: &mut AppState, intent: DraftsIntent) -> Vec<Command> {
    match intent {
        DraftsIntent::ConfirmDelete => confirm_delete(state),
        DraftsIntent::Open | DraftsIntent::AskDelete => Vec::new(),
    }
}

/// Second `D` : ferme la confirmation et demande la suppression ; la liste ne change qu'à la réponse du moteur.
fn confirm_delete(state: &mut AppState) -> Vec<Command> {
    let Popup::ConfirmDeleteDraft { draft, subject } = std::mem::take(&mut state.popup) else {
        return Vec::new();
    };
    state.status.message = Some(format!(
        "Suppression demandée pour « {} »",
        subject_label(&subject)
    ));
    state.drafts.note_delete(draft.clone(), subject);
    vec![Command::Execute {
        tag: Tag::DraftDeleted(draft.clone()),
        request: Request::Compose(ComposeRequest::DeleteDraft { draft }),
    }]
}

/// Réponse à une suppression demandée par la vue : succès annoncé et vue relue ; refus affiché, liste inchangée.
pub fn on_deleted(
    state: &mut AppState,
    subject: &str,
    result: Result<Response, MiraError>,
) -> Vec<Command> {
    match result {
        Ok(_) => {
            let text = format!("Brouillon « {} » supprimé", subject_label(subject));
            state.status.message = Some(text);
            refresh_drafts(state)
        }
        Err(error) => report(state, error),
    }
}
