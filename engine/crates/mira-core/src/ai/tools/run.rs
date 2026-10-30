//! Exécution d'un appel d'outil : aiguillage par nom, sans panique ni erreur interne exposée.

use mira_vocabulary::ToolCall;

use super::render::cut;
use super::run_result::ToolRun;
use super::{list_messages, search_messages};
use crate::api::Engine;

/// Exécute l'appel ; un outil inconnu ou des arguments invalides deviennent un résultat d'erreur.
pub async fn run(engine: &Engine, call: &ToolCall) -> ToolRun {
    match call.name.as_str() {
        search_messages::NAME => search_messages::run(engine, &call.arguments).await,
        list_messages::NAME => list_messages::run(engine, &call.arguments).await,
        other => ToolRun::error(format!("outil inconnu : {}", cut(other, 40))),
    }
}
