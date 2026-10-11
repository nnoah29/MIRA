//! Lots de messages d'un compte en ordre chronologique, pour recalculer les fils.

use mira_vocabulary::{AccountId, MessageSummary, MiraError};
use rusqlite::params;

use crate::storage::Tx;
use crate::storage::repositories::messages::MESSAGE_COLUMNS;

impl Tx<'_> {
    /// Lot de messages d'un compte après la position (date de tri, identifiant), du plus ancien au plus récent
    pub fn messages_for_threading(
        &self,
        account: &AccountId,
        after: Option<(i64, &str)>,
        limit: u32,
    ) -> Result<Vec<(i64, String, MessageSummary)>, MiraError> {
        let sql = format!(
            "SELECT {MESSAGE_COLUMNS} FROM messages m WHERE m.account_id = ?1 AND (?2 IS NULL OR (m.sort_date, m.id) > (?2, ?3)) ORDER BY m.sort_date, m.id LIMIT ?4"
        );
        let key = after.map(|(date, _)| date);
        let id = after.map(|(_, id)| id);
        self.query_summaries(&sql, params![account.as_str(), key, id, i64::from(limit)])
    }
}
