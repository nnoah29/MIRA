//! Liste de messages par vue (boîte, unifiée, non lus, suivis), lue dans la base locale.

use mira_vocabulary::{AccountId, MessageSummary, MiraError, Page};

use super::query::ListQuery;
use crate::storage::Database;

/// Liste une page de messages ; les vues standard portent sur les comptes du profil donnés
pub fn list_messages(
    db: &Database,
    accounts_of_profile: &[AccountId],
    query: &ListQuery,
) -> Result<Page<MessageSummary>, MiraError> {
    db.read(|tx| {
        tx.list_view_messages(
            accounts_of_profile,
            &query.view,
            &query.filters,
            &query.page,
        )
    })
}
