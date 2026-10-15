//! Déroulement du formulaire par le clavier, pour amener l'état à l'étape voulue.
#![allow(dead_code)]

use crossterm::event::KeyCode;
use mira_assembly::mira_vocabulary::ProviderDescriptor;
use mira_tui::commands::{Command, Tag};
use mira_tui::state::AppState;
use mira_tui::update::update;

use super::replies::prepared_reply;
use crate::input_fixtures::{code, key, type_keys};

/// Tape ce texte dans le champ qui a le focus, par la table de touches.
pub fn type_text(state: &mut AppState, text: &str) -> Vec<Command> {
    let keys: Vec<_> = text.chars().map(key).collect();
    type_keys(state, &keys)
}

/// Appuie sur cette touche et rend les commandes produites.
pub fn hit(state: &mut AppState, key_code: KeyCode) -> Vec<Command> {
    type_keys(state, &[code(key_code)])
}

/// Jeton de la requête de préparation ou d'ajout contenue dans ces commandes.
pub fn token_of(commands: &[Command]) -> u64 {
    let tag = commands.iter().find_map(|command| match command {
        Command::Execute { tag, .. } => Some(tag.clone()),
        Command::Quit => None,
    });
    match tag {
        Some(Tag::Prepared(token) | Tag::AccountAdded(token)) => token,
        other => panic!("aucune requête de formulaire : {other:?}"),
    }
}

/// État où le formulaire est ouvert, l'adresse `alice@acme.test` préparée avec ce fournisseur et le focus sur la suite.
pub fn ready_state(descriptor: ProviderDescriptor) -> AppState {
    let mut state = AppState::new(100, 30, 7);
    type_keys(&mut state, &[key('n')]);
    type_text(&mut state, "alice@acme.test");
    let commands = hit(&mut state, KeyCode::Enter);
    let token = token_of(&commands);
    update(&mut state, prepared_reply(token, descriptor));
    state
}
