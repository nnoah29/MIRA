//! Domaine « libellés système » (spec R4) : lecture des poses et choix manuel.

mod assignments;
mod set;

use mira_vocabulary::MiraError;

use crate::api::contract::{Response, SystemLabelRequest as R, SystemLabelRequest};
use crate::api::engine::Engine;

/// Résultat d'une requête du domaine
type Answer = Result<Response, MiraError>;

/// Route une requête du domaine libellés système
pub(crate) async fn route(engine: &Engine, request: SystemLabelRequest) -> Answer {
    match request {
        R::ListSystemLabelAssignments => assignments::handle_list_assignments(engine),
        R::SetSystemLabel { kind, threads, on } => {
            set::handle_set_system_label(engine, kind, &threads, on)
        }
    }
}
