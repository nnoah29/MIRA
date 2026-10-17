//! Enregistrement du corps d'un message préchargé par la synchronisation ; sans effet s'il est déjà présent.

use mira_vocabulary::{ContentState, MessageId, MiraError};

use crate::body_storage::store_parsed_body;
use crate::mime::ParsedMessage;
use crate::storage::Tx;
use crate::sync::context::SyncContext;
use crate::sync::identity::address_text;

/// Enregistre la source brute et son analyse ; sans effet si le corps est déjà présent.
pub fn store_body(
    tx: &Tx<'_>,
    ctx: &SyncContext,
    message: &MessageId,
    raw: &[u8],
    parsed: &ParsedMessage,
) -> Result<bool, MiraError> {
    let Some(summary) = tx.get_message(message)? else {
        return Ok(false);
    };
    if summary.content == ContentState::BodyPresent {
        return Ok(false);
    }
    let addresses = address_text(&summary);
    store_parsed_body(tx, ctx.ids.as_ref(), &summary, raw, parsed, &addresses)?;
    Ok(true)
}
