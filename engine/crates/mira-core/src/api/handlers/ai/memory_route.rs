//! Routage des requêtes de la mémoire de l'IA : une requête, un gestionnaire.

use mira_vocabulary::MiraError;

use super::memory_install::handle_install_memory_model as install_memory;
use crate::api::contract::{AiRequest as R, Response};
use crate::api::engine::Engine;

/// Résultat d'une requête du domaine
type Answer = Result<Response, MiraError>;

/// Route les requêtes de la mémoire
#[rustfmt::skip]
pub(super) async fn route_memory(engine: &Engine, request: R) -> Answer {
    match request {
        R::InstallMemoryModel => install_memory(engine),
        _ => Err(super::unexpected()),
    }
}
