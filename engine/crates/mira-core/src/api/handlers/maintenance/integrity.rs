//! Contrôle d'intégrité de la base du profil.

use std::sync::Arc;

use mira_vocabulary::error::codes::internal;
use mira_vocabulary::{ErrorCategory, MiraError};

use crate::api::contract::{IntegrityReport, Response};
use crate::api::engine::Engine;

/// Lance `PRAGMA integrity_check` sur le pool bloquant et rend le verdict
pub(crate) async fn handle_check_integrity(engine: &Engine) -> Result<Response, MiraError> {
    let database = Arc::clone(&engine.database);
    let ok = tokio::task::spawn_blocking(move || database.read(|tx| tx.integrity_check()))
        .await
        .map_err(|_| {
            MiraError::new(
                ErrorCategory::Internal,
                internal::BUG,
                "contrôle d'intégrité interrompu",
            )
        })??;
    Ok(Response::Integrity(IntegrityReport { ok }))
}
