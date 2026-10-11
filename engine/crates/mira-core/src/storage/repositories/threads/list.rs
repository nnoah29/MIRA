//! Liste paginée des fils d'une boîte.

use mira_vocabulary::{MailboxId, MiraError, Page, PageRequest, Thread, ThreadId};
use rusqlite::params;

use super::rows::{COLUMNS, RawThread};
use crate::storage::Tx;
use crate::storage::errors::storage_error;
use crate::storage::pagination::{cursor_bounds, finish_page};

impl Tx<'_> {
    /// Liste une page de fils ayant des messages dans la boîte, du plus récemment actif au plus ancien
    pub fn list_threads_in_mailbox(
        &self,
        mailbox: &MailboxId,
        request: &PageRequest,
    ) -> Result<Page<Thread>, MiraError> {
        let limit = request.bounded_limit();
        let (key, id) = cursor_bounds(request)?;
        let sql = format!(
            "SELECT {COLUMNS} FROM threads t JOIN messages m ON m.thread_id = t.id JOIN appearances a ON a.message_id = m.id WHERE a.mailbox_id = ?1 AND a.deleted = 0 GROUP BY t.id HAVING (?2 IS NULL OR (MAX(m.sort_date), t.id) < (?2, ?3)) ORDER BY MAX(m.sort_date) DESC, t.id DESC LIMIT ?4"
        );
        let mut statement = self.sql().prepare(&sql).map_err(storage_error)?;
        let args = params![mailbox.as_str(), key, id, i64::from(limit) + 1];
        let raws = statement
            .query_map(args, RawThread::from_row)
            .map_err(storage_error)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(storage_error)?;
        let rows = self.with_participants(raws)?;
        Ok(finish_page(rows, limit))
    }

    /// Complète chaque ligne agrégée avec les participants de son fil
    fn with_participants(
        &self,
        raws: Vec<RawThread>,
    ) -> Result<Vec<(i64, String, Thread)>, MiraError> {
        let mut rows = Vec::with_capacity(raws.len());
        for raw in raws {
            let (last, id) = (raw.last_millis(), raw.thread_id());
            let participants = self.participants_of(&ThreadId::from_raw(&id))?;
            rows.push((last, id, raw.into_thread(participants)));
        }
        Ok(rows)
    }
}
