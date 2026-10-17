//! Recherche plein texte filtrée : FTS5 combiné aux filtres de liste, paginé par date.

use mira_vocabulary::{AccountId, MessageSummary, MiraError, Page, PageRequest};

use super::builder::QueryBuilder;
use super::clauses::{apply_address_filter, apply_filters};
use super::filters::SearchFilters;
use crate::storage::Tx;
use crate::storage::pagination::finish_page;
use crate::storage::repositories::search::escape_match_query;

impl Tx<'_> {
    /// Cherche les mots dans l'index, sur les comptes donnés, avec filtres, du plus récent au plus ancien
    pub fn search_messages_filtered(
        &self,
        accounts: &[AccountId],
        query: &str,
        filters: &SearchFilters,
        request: &PageRequest,
    ) -> Result<Page<MessageSummary>, MiraError> {
        let Some(expression) = escape_match_query(query) else {
            return Ok(finish_page(Vec::new(), request.bounded_limit()));
        };
        let mut builder = QueryBuilder::new("messages m");
        let found = builder.bind(expression);
        builder.require(format!(
            "m.id IN (SELECT message_id FROM messages_fts WHERE messages_fts MATCH {found})"
        ));
        let names: Vec<&str> = accounts.iter().map(AccountId::as_str).collect();
        let list = builder.bind_list(&names);
        builder.require(format!("+m.account_id IN {list}"));
        apply_search_filters(&mut builder, filters);
        self.run_message_page(builder, request)
    }
}

/// Ajoute les filtres communs, le destinataire (à, copie) et la boîte
fn apply_search_filters(builder: &mut QueryBuilder, filters: &SearchFilters) {
    apply_filters(builder, &filters.common);
    if let Some(needle) = &filters.to_contains {
        apply_address_filter(builder, &["m.to_json", "m.cc_json"], needle);
    }
    if let Some(mailbox) = &filters.mailbox {
        let bound = builder.bind(mailbox.as_str().to_string());
        builder.require(format!(
            "EXISTS (SELECT 1 FROM appearances a WHERE a.message_id = m.id AND a.deleted = 0 AND a.mailbox_id = {bound})"
        ));
    }
}
