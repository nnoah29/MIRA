//! Domaine « libellés locaux » (spec R2 §5) : un fichier par opération, écritures contrôlées et idempotentes.

mod assignments;
mod checks;
mod create;
mod delete;
mod idempotency;
mod import;
mod import_apply;
mod list;
mod recolor;
mod rename;
mod set;
mod writing;

use mira_vocabulary::MiraError;

use crate::api::contract::{LocalLabelRequest, LocalLabelRequest as R, Response};
use crate::api::engine::Engine;

/// Résultat d'une requête du domaine
type Answer = Result<Response, MiraError>;

/// Route une requête du domaine libellés locaux
pub(crate) async fn route(engine: &Engine, request: LocalLabelRequest) -> Answer {
    match request {
        R::ListLocalLabels => list::handle_list_local_labels(engine),
        R::ListLocalLabelAssignments => assignments::handle_list_assignments(engine),
        R::DeleteLocalLabel { label } => delete::handle_delete_local_label(engine, &label),
        R::RenameLocalLabel { label, name } => {
            rename::handle_rename_local_label(engine, &label, &name)
        }
        R::RecolorLocalLabel { label, color } => {
            recolor::handle_recolor_local_label(engine, &label, color)
        }
        R::CreateLocalLabel {
            name,
            color,
            idempotency_key,
        } => create::handle_create_local_label(engine, &name, color, idempotency_key),
        R::SetLocalLabel {
            label,
            threads,
            on,
            idempotency_key,
        } => set::handle_set_local_label(engine, &label, &threads, on, idempotency_key),
        R::ImportLocalLabels {
            labels,
            idempotency_key,
        } => import::handle_import_local_labels(engine, &labels, idempotency_key),
    }
}
