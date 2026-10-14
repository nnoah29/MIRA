//! Domaine « maintenance » : volumes, caches, index, intégrité et exports sans secret.

mod diagnostic;
mod integrity;
mod profile_export;
mod purge_cache;
mod rebuild_index;
mod stats;

use mira_vocabulary::MiraError;

use crate::api::contract::{MaintenanceRequest, Response};
use crate::api::engine::Engine;

/// Route une requête du domaine maintenance
pub(crate) async fn route(
    engine: &Engine,
    request: MaintenanceRequest,
) -> Result<Response, MiraError> {
    match request {
        MaintenanceRequest::GetStorageStats => stats::handle_get_storage_stats(engine),
        MaintenanceRequest::PurgeAttachmentCache => purge_cache::handle_purge_cache(engine),
        MaintenanceRequest::RebuildSearchIndex => rebuild_index::handle_rebuild_index(engine),
        MaintenanceRequest::CheckIntegrity => integrity::handle_check_integrity(engine).await,
        MaintenanceRequest::ExportDiagnostic { destination } => {
            diagnostic::handle_export_diagnostic(engine, &destination)
        }
        MaintenanceRequest::ExportProfile { destination } => {
            profile_export::handle_export_profile(engine, &destination)
        }
    }
}
