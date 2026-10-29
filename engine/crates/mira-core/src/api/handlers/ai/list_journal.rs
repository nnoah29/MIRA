//! Lecture du journal des envois à un service d'IA : des comptes et des codes, jamais de texte.

use mira_vocabulary::error::codes::invalid_input;
use mira_vocabulary::{ErrorCategory, MiraError};

use crate::api::contract::{AiResponse, Response};
use crate::api::engine::Engine;

const MAX_LIMIT: u32 = 200;

/// Rend au plus `limit` lignes (1 à 200), les plus récentes d'abord, après la ligne `before` si elle est donnée
pub(crate) fn handle_list_ai_journal(
    engine: &Engine,
    limit: u32,
    before: Option<&str>,
) -> Result<Response, MiraError> {
    if !(1..=MAX_LIMIT).contains(&limit) {
        return Err(MiraError::new(
            ErrorCategory::InvalidInput,
            invalid_input::AI_JOURNAL_LIMIT,
            "la limite du journal doit être comprise entre 1 et 200",
        ));
    }
    let rows = engine
        .database
        .read(|tx| tx.list_ai_journal(limit, before))?;
    Ok(Response::Ai(AiResponse::Journal(rows)))
}
