//! Déplacement du focus et geste « Entrée », qui prépare l'adresse, passe au champ suivant ou envoie.

use super::draft::unavailable_message;
use super::prepare::start_prepare;
use super::submit::submit;
use crate::commands::Command;
use crate::state::AppState;
use crate::state::account_form::{FormField, Phase};

/// Entrée : prépare l'adresse, envoie depuis le mot de passe, sinon passe au champ suivant.
pub fn on_enter(state: &mut AppState) -> Vec<Command> {
    let Some(form) = state.account_form.form() else {
        return Vec::new();
    };
    match (form.phase, form.focus) {
        (Phase::Preparing(_) | Phase::Adding(_), _) => Vec::new(),
        (Phase::Typing, FormField::Address) => start_prepare(state),
        (Phase::Ready, FormField::Address) if form.method_kind().is_none() => explain(state),
        (_, FormField::Password) => submit(state),
        (Phase::Typing | Phase::Ready, _) => step_focus(state, true),
    }
}

/// Passe au champ visible suivant ou précédent ; sans effet pendant une requête.
pub fn step_focus(state: &mut AppState, forward: bool) -> Vec<Command> {
    if let Some(form) = state.account_form.form_mut()
        && matches!(form.phase, Phase::Typing | Phase::Ready)
    {
        form.step_focus(forward);
    }
    Vec::new()
}

/// Dit pourquoi aucune méthode n'est utilisable pour ce fournisseur.
fn explain(state: &mut AppState) -> Vec<Command> {
    if let Some(form) = state.account_form.form_mut() {
        form.error = Some(unavailable_message(form));
    }
    Vec::new()
}
