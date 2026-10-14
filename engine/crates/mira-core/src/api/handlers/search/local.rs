//! Recherche plein texte locale avec filtres (jamais de réseau, invariant 27).

use mira_vocabulary::{MiraError, PageRequest};

use crate::api::contract::Response;
use crate::api::engine::Engine;
use crate::reading::{SearchFilters, search};

/// Cherche dans l'index des comptes du profil et rend une page de résultats
pub(crate) fn handle_search(
    engine: &Engine,
    query: &str,
    filters: &SearchFilters,
    page: &PageRequest,
) -> Result<Response, MiraError> {
    let accounts = engine.account_ids()?;
    let found = search(&engine.database, &accounts, query, filters, page)?;
    Ok(Response::Messages(found))
}
