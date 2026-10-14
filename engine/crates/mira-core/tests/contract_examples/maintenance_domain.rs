//! Exemples du domaine de la maintenance.

use mira_core::api::MaintenanceRequest;

pub const NAMES: &[&str] = &[
    "maintenance.get_storage_stats",
    "maintenance.purge_attachment_cache",
    "maintenance.rebuild_search_index",
    "maintenance.check_integrity",
    "maintenance.export_diagnostic",
    "maintenance.export_profile",
];

pub const EXAMPLES: &[&str] = &[
    r#"{"domain":"maintenance","request":{"op":"get_storage_stats"}}"#,
    r#"{"domain":"maintenance","request":{"op":"purge_attachment_cache"}}"#,
    r#"{"domain":"maintenance","request":{"op":"rebuild_search_index"}}"#,
    r#"{"domain":"maintenance","request":{"op":"check_integrity"}}"#,
    r#"{"domain":"maintenance","request":{"op":"export_diagnostic","destination":"/tmp/diagnostic.json"}}"#,
    r#"{"domain":"maintenance","request":{"op":"export_profile","destination":"/tmp/profile.json"}}"#,
];

/// Nom `domaine.opération` d'une requête du domaine maintenance
pub fn maintenance_name(request: &MaintenanceRequest) -> &'static str {
    match request {
        MaintenanceRequest::GetStorageStats => "maintenance.get_storage_stats",
        MaintenanceRequest::PurgeAttachmentCache => "maintenance.purge_attachment_cache",
        MaintenanceRequest::RebuildSearchIndex => "maintenance.rebuild_search_index",
        MaintenanceRequest::CheckIntegrity => "maintenance.check_integrity",
        MaintenanceRequest::ExportDiagnostic { .. } => "maintenance.export_diagnostic",
        MaintenanceRequest::ExportProfile { .. } => "maintenance.export_profile",
    }
}
