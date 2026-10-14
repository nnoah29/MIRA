//! Frappe et déplacement dans l'éditeur, par intentions.
#![allow(dead_code)]

use crate::navigation_fixtures::press;
use mira_assembly::api::DraftChanges;
use mira_tui::commands::Command;
use mira_tui::state::AppState;
use mira_tui::state::compose::Field;
use mira_tui::update::{ComposeIntent, EditKey, Intent};

/// Applique une intention de rédaction.
pub fn compose(state: &mut AppState, intent: ComposeIntent) -> Vec<Command> {
    press(state, Intent::Compose(intent))
}

/// Applique une touche d'édition.
pub fn edit(state: &mut AppState, key: EditKey) -> Vec<Command> {
    compose(state, ComposeIntent::Edit(key))
}

/// Tape ce texte dans le champ qui a le focus ; un saut de ligne devient `Entrée`.
pub fn type_in_draft(state: &mut AppState, text: &str) {
    for c in text.chars() {
        let key = if c == '\n' {
            EditKey::Newline
        } else {
            EditKey::Char(c)
        };
        edit(state, key);
    }
}

/// Passe au champ suivant jusqu'à avoir le focus sur ce champ.
pub fn focus_on(state: &mut AppState, field: Field) {
    for _ in 0..5 {
        if state.compose.open().map(|c| c.focus) == Some(field) {
            return;
        }
        compose(state, ComposeIntent::NextField);
    }
}

/// Changements qui ne portent que cet objet.
pub fn subject_only(subject: &str) -> DraftChanges {
    DraftChanges {
        subject: Some(subject.to_string()),
        ..DraftChanges::default()
    }
}
