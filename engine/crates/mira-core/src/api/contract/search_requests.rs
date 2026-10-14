//! Requêtes du domaine de la recherche : locale, et étendue au serveur (opération longue).

use mira_vocabulary::{AccountId, OperationId, PageRequest};
use serde::{Deserialize, Serialize};

use crate::reading::SearchFilters;

/// Recherche locale, recherche serveur et lecture du résultat de celle-ci.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum SearchRequest {
    Search {
        query: String,
        filters: SearchFilters,
        page: PageRequest,
    },
    SearchServer {
        account: AccountId,
        query: String,
    },
    GetServerSearchResult {
        operation: OperationId,
    },
}
