//! Liste paginée des fils d'une boîte (lecture locale).

use mira_vocabulary::{MailboxId, MiraError, PageRequest};

use crate::api::contract::Response;
use crate::api::engine::Engine;
use crate::threads::list_threads;

/// Rend une page de fils, du plus récemment actif au plus ancien
pub(crate) fn handle_list_threads(
    engine: &Engine,
    mailbox: &MailboxId,
    page: &PageRequest,
) -> Result<Response, MiraError> {
    Ok(Response::Threads(list_threads(
        &engine.database,
        mailbox,
        page,
    )?))
}
