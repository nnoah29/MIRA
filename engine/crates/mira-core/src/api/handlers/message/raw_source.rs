//! Source brute d'un message stockée en local, rendue en base64.

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use mira_vocabulary::{MessageId, MiraError};

use crate::api::contract::{RawSourceView, Response};
use crate::api::engine::Engine;
use crate::reading::get_raw_source;

/// Rend la source du message, ou `incompatible_state.body_unavailable_offline` si elle est absente
pub(crate) fn handle_get_raw_source(
    engine: &Engine,
    message: &MessageId,
) -> Result<Response, MiraError> {
    let raw = get_raw_source(&engine.database, message)?;
    Ok(Response::RawSource(RawSourceView {
        size: raw.len() as u64,
        content_base64: STANDARD.encode(raw),
    }))
}
