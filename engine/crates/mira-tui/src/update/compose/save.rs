//! Enregistrement (Ctrl+s) : seuls les champs modifiés partent, jamais une adresse invalide.

use mira_assembly::api::{ComposeRequest, Request};

use crate::commands::{Command, Tag};
use crate::state::AppState;
use crate::state::compose::SavePlan;
use crate::view::safe_text::display_safe;

const SEGMENT_CHARS: usize = 40;

/// Envoie les champs modifiés ; signale les adresses invalides et ne fait rien d'autre si rien n'est à envoyer.
pub fn save(state: &mut AppState) -> Vec<Command> {
    let Some(compose) = state.compose.open_mut() else {
        return Vec::new();
    };
    if compose.is_saving() {
        state.status.message = Some("Enregistrement en cours".to_string());
        return Vec::new();
    }
    let plan = compose.plan_save();
    compose.begin_save(&plan);
    if plan.sent.is_empty() {
        state.status.message = Some(nothing_to_send(&plan));
        return Vec::new();
    }
    let draft = compose.draft().clone();
    vec![Command::Execute {
        tag: Tag::DraftSaved(draft.clone()),
        request: Request::Compose(ComposeRequest::UpdateDraft {
            draft,
            changes: plan.changes,
        }),
    }]
}

/// Pourquoi rien ne part : première adresse invalide (neutralisée, raccourcie), sinon rien de modifié.
fn nothing_to_send(plan: &SavePlan) -> String {
    match plan.invalid.first() {
        Some((field, segment)) => {
            let shown: String = segment.chars().take(SEGMENT_CHARS).collect();
            format!(
                "Enregistrement : adresse invalide dans {} : « {} »",
                field.label(),
                display_safe(&shown)
            )
        }
        None => "Rien à enregistrer".to_string(),
    }
}
