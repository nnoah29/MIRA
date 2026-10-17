//! Liste paginée de messages d'une boîte ou d'une vue standard (lecture locale).

use mira_vocabulary::MiraError;

use crate::api::contract::Response;
use crate::api::engine::Engine;
use crate::reading::{ListQuery, list_messages};

/// Rend une page de messages de la vue demandée, sur les comptes du profil
pub(crate) fn handle_list_messages(
    engine: &Engine,
    query: &ListQuery,
) -> Result<Response, MiraError> {
    let accounts = engine.account_ids()?;
    let page = list_messages(&engine.database, &accounts, query)?;
    Ok(Response::Messages(page))
}
