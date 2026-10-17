//! Validation du formulaire d'export : un chemin saisi, jamais de chemin par défaut.

use mira_assembly::api::{MaintenanceRequest, Request};

use super::operation::{say, submit_operation};
use crate::commands::Command;
use crate::state::AppState;
use crate::state::settings::{ExportKind, Form, Operation};

/// Lance l'export vers le fichier saisi ; un chemin vide laisse le formulaire ouvert et ne part pas.
pub fn submit_export(state: &mut AppState, form: &Form, kind: ExportKind) -> Vec<Command> {
    let destination = form.value(0).trim().to_string();
    if destination.is_empty() {
        return say(state, "Indiquez le fichier à écrire");
    }
    let (operation, request) = match kind {
        ExportKind::Diagnostic => (
            Operation::ExportDiagnostic,
            MaintenanceRequest::ExportDiagnostic { destination },
        ),
        ExportKind::Profile => (
            Operation::ExportProfile,
            MaintenanceRequest::ExportProfile { destination },
        ),
    };
    submit_operation(state, operation, Request::Maintenance(request))
}
