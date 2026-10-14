//! Contrôle d'intégrité de la base du profil.

use mira_vocabulary::MiraError;

use crate::api::contract::{IntegrityReport, Response};
use crate::api::engine::Engine;

/// Lance `PRAGMA integrity_check` et rend le verdict
pub(crate) fn handle_check_integrity(engine: &Engine) -> Result<Response, MiraError> {
    let ok = engine.database.read(|tx| tx.integrity_check())?;
    Ok(Response::Integrity(IntegrityReport { ok }))
}
