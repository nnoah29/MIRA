//! Export d'un diagnostic sans secret ni contenu de message.

use std::path::Path;

use mira_vocabulary::error::codes::local_storage;
use mira_vocabulary::{ErrorCategory, MiraError};

use crate::api::contract::{ExportReport, Response};
use crate::api::engine::Engine;
use crate::maintenance::{build_diagnostic, write_export_file};

/// Écrit le diagnostic en JSON à la destination, de façon atomique
pub(crate) fn handle_export_diagnostic(
    engine: &Engine,
    destination: &str,
) -> Result<Response, MiraError> {
    let report = build_diagnostic(&engine.database, &engine.config.profile)?;
    let bytes = serde_json::to_vec_pretty(&report).map_err(|_| {
        MiraError::new(
            ErrorCategory::LocalStorage,
            local_storage::IO,
            "le diagnostic n'a pas pu être sérialisé",
        )
    })?;
    write_export_file(Path::new(destination), &bytes)?;
    Ok(Response::Exported(ExportReport {
        path: destination.to_string(),
        bytes: bytes.len() as u64,
    }))
}
