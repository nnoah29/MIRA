//! Arrêt sans `q` (signal, clavier fermé) : les modifications du brouillon ouvert partent, le brouillon n'est jamais supprimé.

use mira_assembly::api::{ComposeRequest, Request};

use crate::commands::{Command, Tag};
use crate::state::AppState;

/// Enregistre les champs modifiés du brouillon ouvert (et rien d'autre) avant l'arrêt.
pub fn shutdown_commands(state: &mut AppState) -> Vec<Command> {
    let Some(compose) = state.compose.open_mut() else {
        return Vec::new();
    };
    if !compose.has_dirty_fields() {
        return Vec::new();
    }
    let plan = compose.plan_save();
    if plan.sent.is_empty() {
        return Vec::new();
    }
    compose.begin_save(&plan);
    let draft = compose.draft().clone();
    vec![Command::Execute {
        tag: Tag::DraftSaved(draft.clone()),
        request: Request::Compose(ComposeRequest::UpdateDraft {
            draft,
            changes: plan.changes,
        }),
    }]
}
