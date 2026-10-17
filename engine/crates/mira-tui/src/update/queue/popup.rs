//! Décision sur un envoi incertain : renvoyer ou non, choisi explicitement par l'utilisateur.

use mira_assembly::api::{ComposeRequest, Request};

use super::change::change;
use super::intent::QueueIntent;
use crate::commands::Command;
use crate::state::{AppState, Popup};

/// Applique `r` (renvoyer) ou `n` (ne pas renvoyer) ; sans fenêtre de décision ouverte, rien ne se passe.
pub fn route(state: &mut AppState, intent: QueueIntent) -> Vec<Command> {
    match intent {
        QueueIntent::Resend => decide(state, true),
        QueueIntent::DontResend => decide(state, false),
        QueueIntent::Open | QueueIntent::Retry | QueueIntent::Cancel | QueueIntent::Resolve => {
            Vec::new()
        }
    }
}

/// Ferme la fenêtre et transmet la décision au moteur.
fn decide(state: &mut AppState, resend: bool) -> Vec<Command> {
    let Popup::ResolveUncertain { action, .. } = &state.popup else {
        return Vec::new();
    };
    let action = action.clone();
    state.popup = Popup::Closed;
    state.status.message = Some(if resend {
        "Renvoi demandé".to_string()
    } else {
        "Envoi abandonné : le brouillon revient en édition (rouvrez-le avec B)".to_string()
    });
    let request = ComposeRequest::ResolveUncertainSend {
        action: action.clone(),
        resend,
    };
    change(action, Request::Compose(request))
}
