//! Opérations de maintenance qui détruisent ou écrivent : confirmation ou chemin saisi avant tout envoi.

use mira_assembly::api::{MaintenanceRequest, Request};

use super::operation::submit_operation;
use crate::commands::Command;
use crate::state::AppState;
use crate::state::settings::{Ask, Dialog, ExportKind, Field, Form, FormKind, Operation};

/// Ouvre la confirmation de cette opération ; rien ne part avant la confirmation.
pub fn ask(state: &mut AppState, ask: Ask) -> Vec<Command> {
    state.settings.dialog = Dialog::Confirm(ask);
    Vec::new()
}

/// Ouvre la saisie du fichier à écrire ; rien ne part avant Entrée sur un chemin non vide.
pub fn ask_export(state: &mut AppState, kind: ExportKind) -> Vec<Command> {
    let fields = vec![Field::new("Fichier à écrire", "")];
    state.settings.dialog = Dialog::Form(Form::new(FormKind::Export(kind), fields));
    Vec::new()
}

/// Confirmation reçue : lance l'opération et ferme la fenêtre, si aucune autre n'est en cours.
pub fn confirm_ask(state: &mut AppState) -> Vec<Command> {
    let Dialog::Confirm(asked) = state.settings.dialog else {
        return Vec::new();
    };
    let (operation, request) = match asked {
        Ask::PurgeCache => (Operation::PurgeCache, MaintenanceRequest::PurgeAttachmentCache),
        Ask::RebuildIndex => (Operation::RebuildIndex, MaintenanceRequest::RebuildSearchIndex),
    };
    submit_operation(state, operation, Request::Maintenance(request))
}
