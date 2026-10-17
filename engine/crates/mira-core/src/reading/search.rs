//! Recherche locale : plein texte FTS5 et filtres, sans réseau.

use mira_vocabulary::{AccountId, MessageSummary, MiraError, Page, PageRequest};

use super::query::SearchFilters;
use crate::storage::Database;

/// Cherche dans l'index local, sur les comptes du profil donnés, du plus récent au plus ancien
pub fn search(
    db: &Database,
    accounts_of_profile: &[AccountId],
    query: &str,
    filters: &SearchFilters,
    page: &PageRequest,
) -> Result<Page<MessageSummary>, MiraError> {
    db.read(|tx| tx.search_messages_filtered(accounts_of_profile, query, filters, page))
}
