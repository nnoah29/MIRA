//! Export du profil en JSON (D-29) : configuration et intentions, sans secret ni miroir.

use std::path::Path;

use mira_vocabulary::error::codes::local_storage;
use mira_vocabulary::{ErrorCategory, MiraError};

use crate::api::contract::{ExportReport, Response};
use crate::api::engine::Engine;
use crate::maintenance::{build_profile_export, write_export_file};

/// Écrit l'export du profil à la destination, de façon atomique
pub(crate) fn handle_export_profile(
    engine: &Engine,
    destination: &str,
) -> Result<Response, MiraError> {
    let export = build_profile_export(&engine.database, &engine.config.profile)?;
    let bytes = serde_json::to_vec_pretty(&export).map_err(|_| {
        MiraError::new(
            ErrorCategory::LocalStorage,
            local_storage::IO,
            "l'export du profil n'a pas pu être sérialisé",
        )
    })?;
    write_export_file(Path::new(destination), &bytes)?;
    Ok(Response::Exported(ExportReport {
        path: destination.to_string(),
        bytes: bytes.len() as u64,
    }))
}
