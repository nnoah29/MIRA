//! Page Maintenance : volumes du miroir, contrôle d'intégrité et routage des opérations.

use mira_assembly::api::{MaintenanceRequest, Request, Response};
use mira_assembly::mira_vocabulary::MiraError;

use super::intent::MaintenanceOp;
use super::maintenance_ask::{ask, ask_export};
use super::operation::start_operation;
use crate::commands::{Command, SettingsTag, Tag};
use crate::state::AppState;
use crate::state::settings::{Ask, Dialog, ExportKind, Operation, SettingsTab};
use crate::update::accounts_reply::{report, unexpected};

/// Applique une opération de la page Maintenance ; sur une autre page ou sous une fenêtre elle est ignorée.
pub fn on_maintenance_op(state: &mut AppState, op: MaintenanceOp) -> Vec<Command> {
    if state.settings.tab != SettingsTab::Maintenance || state.settings.dialog != Dialog::None {
        return Vec::new();
    }
    match op {
        MaintenanceOp::Stats => load_stats(state),
        MaintenanceOp::Check => check_integrity(state),
        MaintenanceOp::Purge => ask(state, Ask::PurgeCache),
        MaintenanceOp::Rebuild => ask(state, Ask::RebuildIndex),
        MaintenanceOp::ExportDiagnostic => ask_export(state, ExportKind::Diagnostic),
        MaintenanceOp::ExportProfile => ask_export(state, ExportKind::Profile),
    }
}

/// Lit les volumes du miroir.
pub fn load_stats(state: &mut AppState) -> Vec<Command> {
    let token = state.settings.begin_stats();
    vec![Command::Execute {
        tag: Tag::Settings(SettingsTag::Stats(token)),
        request: Request::Maintenance(MaintenanceRequest::GetStorageStats),
    }]
}

/// Contrôle l'intégrité de la base (lecture seule, sans confirmation).
fn check_integrity(state: &mut AppState) -> Vec<Command> {
    let request = Request::Maintenance(MaintenanceRequest::CheckIntegrity);
    start_operation(state, Operation::CheckIntegrity, request)
}

/// Réponse à la lecture des volumes : prise seulement si c'est la lecture attendue de la vue ouverte.
pub fn on_stats(
    state: &mut AppState,
    token: u64,
    result: Result<Response, MiraError>,
) -> Vec<Command> {
    if !state.settings.take_stats(token) {
        return Vec::new();
    }
    match result {
        Ok(Response::StorageStats(stats)) => state.settings.maintenance.stats = Some(stats),
        Ok(_) => return unexpected(state),
        Err(error) => return report(state, error),
    }
    Vec::new()
}
