//! Domaine « recherche » : locale (instantanée) et étendue au serveur (opération longue).

mod local;
mod server;
mod server_result;

use mira_vocabulary::MiraError;

use crate::api::contract::{Response, SearchRequest};
use crate::api::engine::Engine;

/// Route une requête du domaine recherche
pub(crate) async fn route(engine: &Engine, request: SearchRequest) -> Result<Response, MiraError> {
    match request {
        SearchRequest::Search {
            query,
            filters,
            page,
        } => local::handle_search(engine, &query, &filters, &page),
        SearchRequest::SearchServer { account, query } => {
            server::handle_search_server(engine, &account, query)
        }
        SearchRequest::GetServerSearchResult { operation } => {
            server_result::handle_get_server_search_result(engine, &operation)
        }
    }
}
