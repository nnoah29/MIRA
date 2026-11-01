//! Liste des conversations enregistrées : métadonnées seules, les plus récentes d'abord.

use mira_vocabulary::error::codes::invalid_input;
use mira_vocabulary::{ErrorCategory, MiraError};

use crate::api::contract::{AiResponse, Response};
use crate::api::engine::Engine;

const MAX_LIMIT: u32 = 200;

/// Rend au plus `limit` conversations (1 à 200), après la conversation `before` si elle est donnée
pub(crate) fn handle_list_ai_conversations(
    engine: &Engine,
    limit: u32,
    before: Option<&str>,
) -> Result<Response, MiraError> {
    if !(1..=MAX_LIMIT).contains(&limit) {
        return Err(MiraError::new(
            ErrorCategory::InvalidInput,
            invalid_input::PARAMETER,
            "la limite des conversations doit être comprise entre 1 et 200",
        ));
    }
    let items = engine
        .database
        .read(|tx| tx.list_ai_conversations(limit, before))?;
    Ok(Response::Ai(AiResponse::Conversations(items)))
}
