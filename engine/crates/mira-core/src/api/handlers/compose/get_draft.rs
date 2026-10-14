//! Lecture d'un brouillon (lecture locale).

use mira_vocabulary::{DraftId, MiraError};

use crate::actions::errors::not_found::draft_not_found;
use crate::api::contract::{Response, WriteResponse};
use crate::api::engine::Engine;

/// Rend le brouillon, ou `not_found.draft`
pub(crate) fn handle_get_draft(engine: &Engine, draft: &DraftId) -> Result<Response, MiraError> {
    let found = engine
        .database
        .read(|tx| tx.get_draft(draft))?
        .ok_or_else(|| draft_not_found(draft))?;
    Ok(Response::Write(WriteResponse::Draft(Box::new(found))))
}
