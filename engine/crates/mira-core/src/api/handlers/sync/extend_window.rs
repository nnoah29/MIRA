//! « Remonter plus loin » (D-21, S12) : télécharge les en-têtes d'une période plus ancienne, opération annulable.

use mira_vocabulary::error::codes::invalid_input;
use mira_vocabulary::{ErrorCategory, MailboxId, MiraError};

use super::targets::mailbox_owner;
use crate::api::contract::Response;
use crate::api::engine::Engine;

/// Refuse un nombre de jours nul
fn require_days(days: u32) -> Result<(), MiraError> {
    if days > 0 {
        return Ok(());
    }
    Err(MiraError::new(
        ErrorCategory::InvalidInput,
        invalid_input::PARAMETER,
        "le nombre de jours doit être positif",
    ))
}

/// Lance l'extension de la plage de la boîte et rend l'opération (annulable, progression par événements)
pub(crate) fn handle_extend_window(
    engine: &Engine,
    mailbox: MailboxId,
    days: u32,
) -> Result<Response, MiraError> {
    require_days(days)?;
    let account = mailbox_owner(engine, &mailbox)?;
    let operation = engine
        .sync
        .start_extension(&account, &engine.operations, mailbox, days)?;
    Ok(Response::Operation(engine.operations.get(&operation)?))
}
