//! Routage des réponses du moteur aux requêtes de la vue Paramètres, par étiquette.

use mira_assembly::api::Response;
use mira_assembly::mira_vocabulary::MiraError;

use super::events_reply::on_events;
use super::identity_actions::on_identities;
use super::maintenance::on_stats;
use super::operation_reply::on_operation;
use crate::commands::{Command, SettingsTag};
use crate::state::AppState;

/// Traite la réponse selon l'étiquette de la demande ; chaque handler vérifie son jeton.
pub fn on_settings(
    state: &mut AppState,
    tag: SettingsTag,
    result: Result<Response, MiraError>,
) -> Vec<Command> {
    match tag {
        SettingsTag::Identities(token) => on_identities(state, token, result),
        SettingsTag::Events(token) => on_events(state, token, result),
        SettingsTag::Stats(token) => on_stats(state, token, result),
        SettingsTag::Operation(token) => on_operation(state, token, result),
    }
}
