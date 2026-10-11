//! Lectures sur les messages : un message, une page d'une boîte.

use mira_vocabulary::{MailboxId, MessageId, MessageSummary, MiraError, Page, PageRequest};
use rusqlite::params;

use super::rows::COLUMNS;
use crate::storage::Tx;
use crate::storage::pagination::{cursor_bounds, finish_page};

impl Tx<'_> {
    /// Relit un message avec les boîtes où il apparaît, ou rien s'il est inconnu
    pub fn get_message(&self, id: &MessageId) -> Result<Option<MessageSummary>, MiraError> {
        let sql = format!("SELECT {COLUMNS} FROM messages m WHERE m.id = ?1");
        let mut found = self.query_summaries(&sql, [id.as_str()])?;
        Ok(found.pop().map(|(_, _, summary)| summary))
    }

    /// Liste une page de messages d'une boîte, du plus récent au plus ancien
    pub fn list_messages_in_mailbox(
        &self,
        mailbox: &MailboxId,
        request: &PageRequest,
    ) -> Result<Page<MessageSummary>, MiraError> {
        let limit = request.bounded_limit();
        let (key, id) = cursor_bounds(request)?;
        let sql = format!(
            "SELECT {COLUMNS} FROM messages m JOIN appearances a ON a.message_id = m.id WHERE a.mailbox_id = ?1 AND a.deleted = 0 AND (?2 IS NULL OR (m.sort_date, m.id) < (?2, ?3)) ORDER BY m.sort_date DESC, m.id DESC LIMIT ?4"
        );
        let args = params![mailbox.as_str(), key, id, i64::from(limit) + 1];
        let rows = self.query_summaries(&sql, args)?;
        Ok(finish_page(rows, limit))
    }
}
