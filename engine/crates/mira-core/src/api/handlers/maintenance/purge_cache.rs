//! Purge du cache des pièces jointes téléchargées.

use mira_vocabulary::MiraError;

use crate::api::contract::Response;
use crate::api::engine::Engine;
use crate::maintenance::purge_attachment_cache;

/// Supprime les fichiers du cache ; les pièces jointes redeviennent « non téléchargées »
pub(crate) fn handle_purge_cache(engine: &Engine) -> Result<Response, MiraError> {
    purge_attachment_cache(&engine.database, &engine.paths.attachments_cache)?;
    Ok(Response::Done)
}
