//! Rendu humain des réponses de maintenance : volumes, intégrité, exports.

use mira_assembly::api::Response;
use mira_assembly::api::contract::{ExportReport, IntegrityReport};
use mira_assembly::storage::repositories::maintenance::StorageStats;

use super::generic::pretty_json;

/// Volumes du miroir, un par ligne.
fn stats_text(stats: &StorageStats) -> String {
    format!(
        "Boîtes         : {}\nMessages       : {}\nApparitions    : {}\nFils           : {}\nSources brutes : {} ({} octets, {} compressés)\nPièces jointes : {} ({} en cache)",
        stats.mailboxes,
        stats.messages,
        stats.appearances,
        stats.threads,
        stats.raw_sources,
        stats.raw_bytes,
        stats.compressed_bytes,
        stats.attachments,
        stats.attachments_cached
    )
}

/// Verdict du contrôle d'intégrité.
fn integrity_text(report: &IntegrityReport) -> String {
    if report.ok {
        "base intègre".to_string()
    } else {
        "base endommagée".to_string()
    }
}

/// Fichier d'export écrit et sa taille.
fn exported_text(report: &ExportReport) -> String {
    format!("export écrit : {} ({} octets)", report.path, report.bytes)
}

/// Texte d'une réponse de maintenance ; les autres formes retombent sur le JSON du contrat.
pub fn maintenance_text(response: &Response) -> String {
    match response {
        Response::StorageStats(stats) => stats_text(stats),
        Response::Integrity(report) => integrity_text(report),
        Response::Exported(report) => exported_text(report),
        other => pretty_json(other),
    }
}
