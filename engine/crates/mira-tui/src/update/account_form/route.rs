//! Routage des intentions : hors formulaire seule l'ouverture agit ; formulaire ouvert, il reçoit tout (modal).

use super::edit::apply_edit;
use super::focus::{on_enter, step_focus};
use super::intent::AccountFormIntent;
use super::open::{confirm_close, keep_editing, open_form, request_close};
use super::servers::cycle_security;
use crate::commands::Command;
use crate::state::AppState;
use crate::update::{Intent, panes};

/// Intention du formulaire reçue hors formulaire : seule l'ouverture agit.
pub fn route_idle(state: &mut AppState, intent: AccountFormIntent) -> Vec<Command> {
    match intent {
        AccountFormIntent::Open => open_form(state),
        AccountFormIntent::Edit(_)
        | AccountFormIntent::Enter
        | AccountFormIntent::NextField
        | AccountFormIntent::PrevField
        | AccountFormIntent::CycleSecurity
        | AccountFormIntent::Close
        | AccountFormIntent::ConfirmClose
        | AccountFormIntent::KeepEditing => Vec::new(),
    }
}

/// Formulaire ouvert : seules ses intentions et la sortie agissent ; les autres sont ignorées.
#[rustfmt::skip]
pub fn route_open(state: &mut AppState, intent: Intent) -> Vec<Command> {
    match intent {
        Intent::AccountForm(form) => on_form_intent(state, form),
        Intent::Quit => panes::quit(state),
        Intent::MoveDown | Intent::MoveUp | Intent::Top | Intent::Bottom | Intent::Open | Intent::Back
        | Intent::NextPane | Intent::PrevPane | Intent::NextAccount | Intent::PrevAccount | Intent::Archive
        | Intent::Trash | Intent::DeletePermanently | Intent::ToggleFlag | Intent::ToggleRead
        | Intent::OpenMovePicker | Intent::Undo | Intent::Confirm | Intent::Cancel | Intent::Search(_)
        | Intent::Compose(_) | Intent::Queue(_) | Intent::Drafts(_) | Intent::StartSync | Intent::Help
        | Intent::Palette(_) | Intent::Settings(_) => Vec::new(),
    }
}

/// Applique une intention au formulaire ouvert ; sous la confirmation de fermeture, seules les réponses à cette question comptent.
fn on_form_intent(state: &mut AppState, intent: AccountFormIntent) -> Vec<Command> {
    let confirming = state
        .account_form
        .form()
        .is_some_and(|form| form.confirm_close);
    let answers_close = matches!(
        intent,
        AccountFormIntent::ConfirmClose | AccountFormIntent::KeepEditing | AccountFormIntent::Close
    );
    if confirming && !answers_close {
        return Vec::new();
    }
    match intent {
        AccountFormIntent::Open => Vec::new(),
        AccountFormIntent::Edit(key) => apply_edit(state, key),
        AccountFormIntent::Enter => on_enter(state),
        AccountFormIntent::NextField => step_focus(state, true),
        AccountFormIntent::PrevField => step_focus(state, false),
        AccountFormIntent::CycleSecurity => cycle_security(state),
        AccountFormIntent::Close => request_close(state),
        AccountFormIntent::ConfirmClose => confirm_close(state),
        AccountFormIntent::KeepEditing => keep_editing(state),
    }
}
