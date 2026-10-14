//! Exemples des domaines des fils et de la recherche.

use mira_core::api::{SearchRequest, ThreadRequest};

pub const NAMES: &[&str] = &[
    "thread.list_threads",
    "thread.get_thread",
    "search.search",
    "search.search_server",
    "search.get_server_search_result",
];

pub const EXAMPLES: &[&str] = &[
    r#"{"domain":"thread","request":{"op":"list_threads","mailbox":"m","page":{"cursor":null,"limit":20}}}"#,
    r#"{"domain":"thread","request":{"op":"get_thread","thread":"t"}}"#,
    r#"{"domain":"search","request":{"op":"search","query":"facture","filters":{"common":{"unread_only":false,"flagged_only":false,"has_attachments":false,"from_contains":null,"since":null,"until":null},"to_contains":null,"mailbox":null},"page":{"cursor":null,"limit":50}}}"#,
    r#"{"domain":"search","request":{"op":"search_server","account":"a","query":"facture"}}"#,
    r#"{"domain":"search","request":{"op":"get_server_search_result","operation":"o"}}"#,
];

/// Nom `domaine.opération` d'une requête du domaine fils
pub fn thread_name(request: &ThreadRequest) -> &'static str {
    match request {
        ThreadRequest::ListThreads { .. } => "thread.list_threads",
        ThreadRequest::GetThread { .. } => "thread.get_thread",
    }
}

/// Nom `domaine.opération` d'une requête du domaine recherche
pub fn search_name(request: &SearchRequest) -> &'static str {
    match request {
        SearchRequest::Search { .. } => "search.search",
        SearchRequest::SearchServer { .. } => "search.search_server",
        SearchRequest::GetServerSearchResult { .. } => "search.get_server_search_result",
    }
}
