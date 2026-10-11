//! Recherche plein texte : requête utilisateur échappée, résultats paginés par date.

use mira_vocabulary::{AccountId, MessageSummary, MiraError, Page, PageRequest};
use rusqlite::params;

use crate::storage::Tx;
use crate::storage::pagination::{cursor_bounds, finish_page};
use crate::storage::repositories::messages::MESSAGE_COLUMNS;

/// Échappe une requête libre : chaque mot devient une phrase entre guillemets doubles (aucune syntaxe FTS5 injectable)
pub(crate) fn escape_match_query(query: &str) -> Option<String> {
    let words: Vec<String> = query
        .split_whitespace()
        .map(|word| format!("\"{}\"", word.replace('"', "\"\"")))
        .collect();
    if words.is_empty() {
        return None;
    }
    Some(words.join(" "))
}

impl Tx<'_> {
    /// Cherche les messages dont l'index contient tous les mots, du plus récent au plus ancien
    pub fn search_messages(
        &self,
        account: Option<&AccountId>,
        query: &str,
        request: &PageRequest,
    ) -> Result<Page<MessageSummary>, MiraError> {
        let limit = request.bounded_limit();
        let (key, id) = cursor_bounds(request)?;
        let Some(expression) = escape_match_query(query) else {
            return Ok(finish_page(Vec::new(), limit));
        };
        let sql = format!(
            "SELECT {MESSAGE_COLUMNS} FROM messages m WHERE m.id IN (SELECT message_id FROM messages_fts WHERE messages_fts MATCH ?1) AND (?2 IS NULL OR m.account_id = ?2) AND (?3 IS NULL OR (m.sort_date, m.id) < (?3, ?4)) ORDER BY m.sort_date DESC, m.id DESC LIMIT ?5"
        );
        let scope = account.map(AccountId::as_str);
        let args = params![expression, scope, key, id, i64::from(limit) + 1];
        Ok(finish_page(self.query_summaries(&sql, args)?, limit))
    }
}
