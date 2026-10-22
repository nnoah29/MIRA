//! Suppression d'un libellé local et de ses poses.

use mira_vocabulary::{LocalLabelId, MiraError};

use super::writing::{announce, done};
use crate::api::contract::Response;
use crate::api::engine::Engine;
use crate::labels::refresh_inbox_counters;

/// Supprime le libellé et ses poses ; inconnu : `not_found.local_label`
pub(crate) fn handle_delete_local_label(
    engine: &Engine,
    label: &LocalLabelId,
) -> Result<Response, MiraError> {
    engine.database.write(|tx| tx.delete_local_label(label))?;
    announce(engine)?;
    refresh_inbox_counters(&engine.database, &engine.events, false)?;
    Ok(done())
}
