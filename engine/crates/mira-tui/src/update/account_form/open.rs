//! Ouverture et fermeture du formulaire ; la fermeture d'une saisie commencée demande confirmation.

use crate::commands::Command;
use crate::state::AppState;
use crate::state::account_form::Phase;

/// Ouvre un formulaire vide.
pub fn open_form(state: &mut AppState) -> Vec<Command> {
    state.account_form.open();
    state.status.message = None;
    Vec::new()
}

/// Échap : ferme tout de suite un formulaire vide, sinon demande confirmation ; sous la confirmation, il y répond « continuer ».
pub fn request_close(state: &mut AppState) -> Vec<Command> {
    let Some(form) = state.account_form.form_mut() else {
        return Vec::new();
    };
    if form.confirm_close {
        form.confirm_close = false;
    } else if form.has_input() {
        form.confirm_close = true;
    } else {
        state.account_form.close();
    }
    Vec::new()
}

/// Ferme le formulaire après confirmation ; un ajout encore en cours n'est plus suivi à l'écran.
pub fn confirm_close(state: &mut AppState) -> Vec<Command> {
    let Some(form) = state.account_form.form() else {
        return Vec::new();
    };
    if !form.confirm_close {
        return Vec::new();
    }
    if matches!(form.phase, Phase::Adding(_)) {
        state.status.message =
            Some("Ajout abandonné à l'écran : le compte peut quand même être ajouté".to_string());
    }
    state.account_form.close();
    Vec::new()
}

/// Revient à la saisie sans rien perdre.
pub fn keep_editing(state: &mut AppState) -> Vec<Command> {
    if let Some(form) = state.account_form.form_mut() {
        form.confirm_close = false;
    }
    Vec::new()
}
