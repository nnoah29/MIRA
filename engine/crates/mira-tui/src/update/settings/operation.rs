//! Lancement d'une modification : une seule à la fois, suivie par un jeton.

use mira_assembly::api::Request;

use crate::commands::{Command, SettingsTag, Tag};
use crate::state::AppState;
use crate::state::settings::{Dialog, Operation};

/// Montre ce message dans la barre d'état, sans rien envoyer.
pub fn say(state: &mut AppState, text: &str) -> Vec<Command> {
    state.status.message = Some(text.to_string());
    Vec::new()
}

/// Lance la requête sous un jeton neuf ; si une autre modification est en cours, rien ne part.
pub fn start_operation(
    state: &mut AppState,
    operation: Operation,
    request: Request,
) -> Vec<Command> {
    let Some(token) = state.settings.begin_operation(operation) else {
        return say(state, "Une opération est déjà en cours");
    };
    state.status.message = Some("Opération en cours…".to_string());
    vec![Command::Execute {
        tag: Tag::Settings(SettingsTag::Operation(token)),
        request,
    }]
}

/// Lance la modification demandée par le formulaire ouvert, qui ne se ferme que si elle part.
pub fn submit_operation(
    state: &mut AppState,
    operation: Operation,
    request: Request,
) -> Vec<Command> {
    let commands = start_operation(state, operation, request);
    if !commands.is_empty() {
        state.settings.dialog = Dialog::None;
    }
    commands
}
