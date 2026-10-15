//! Brouillon visé par une touche de la vue : toujours celui sélectionné, jamais un autre s'il a disparu.

use mira_assembly::mira_vocabulary::{Draft, DraftState};

use crate::state::AppState;
use crate::view::safe_text::display_safe;

const SUBJECT_CHARS: usize = 60;

/// Brouillon sélectionné ; s'il a disparu d'une relecture, le dit et ne vise rien d'autre.
pub fn selected_draft(state: &mut AppState) -> Option<Draft> {
    let draft = state.drafts.selected().cloned();
    if draft.is_none() && !state.drafts.items().is_empty() {
        state.status.message = Some("Ce brouillon n'existe plus".to_string());
    }
    draft
}

/// Pourquoi ce brouillon ne peut pas être rouvert (planifié ou en cours d'envoi) ; rien s'il est en édition.
pub fn not_editable_reason(draft: &Draft) -> Option<&'static str> {
    match draft.state {
        DraftState::Editing => None,
        DraftState::Scheduled { .. } => Some(
            "Brouillon planifié pour l'envoi : annulez l'envoi dans la file (Q) pour le modifier",
        ),
        DraftState::Sending => Some("Brouillon en cours d'envoi : il ne peut plus être modifié"),
    }
}

/// Objet du brouillon pour un message : raccourci et neutralisé, « (sans objet) » s'il est vide.
pub fn subject_label(subject: &str) -> String {
    let cut: String = subject.chars().take(SUBJECT_CHARS).collect();
    let shown = display_safe(cut.trim());
    if shown.is_empty() {
        "(sans objet)".to_string()
    } else {
        shown
    }
}

/// Montre ce message dans la barre d'état, sans rien envoyer.
pub fn say(state: &mut AppState, text: &str) -> Vec<crate::commands::Command> {
    state.status.message = Some(text.to_string());
    Vec::new()
}
