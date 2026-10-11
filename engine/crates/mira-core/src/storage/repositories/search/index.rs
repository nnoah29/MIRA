//! Alimentation de l'index plein texte : indexer, désindexer, reconstruire.

use mira_vocabulary::{MessageId, MiraError};
use rusqlite::params;

use crate::storage::Tx;
use crate::storage::errors::storage_error;

const ADDRESS_TEXT: &str = "COALESCE((SELECT group_concat(COALESCE(json_extract(j.value, '$.name'), '') || ' ' || json_extract(j.value, '$.address'), ' ') FROM (SELECT value FROM json_each(m.from_json) UNION ALL SELECT value FROM json_each(m.to_json) UNION ALL SELECT value FROM json_each(m.cc_json)) j), '')";

impl Tx<'_> {
    /// Indexe (ou réindexe) un message : sujet, adresses et corps
    pub fn index_message(
        &self,
        message: &MessageId,
        subject: &str,
        addresses: &str,
        body: &str,
    ) -> Result<(), MiraError> {
        self.unindex_message(message)?;
        self.sql()
            .execute(
                "INSERT INTO messages_fts (message_id, subject, addresses, body) VALUES (?1, ?2, ?3, ?4)",
                params![message.as_str(), subject, addresses, body],
            )
            .map_err(storage_error)?;
        Ok(())
    }

    /// Retire un message de l'index (sans effet s'il n'y est pas)
    pub fn unindex_message(&self, message: &MessageId) -> Result<(), MiraError> {
        self.sql()
            .execute(
                "DELETE FROM messages_fts WHERE message_id = ?1",
                [message.as_str()],
            )
            .map_err(storage_error)?;
        Ok(())
    }

    /// Reconstruit tout l'index depuis les messages et le cache du corps (aperçu à défaut)
    pub fn rebuild_search_index(&self) -> Result<(), MiraError> {
        self.sql()
            .execute("DELETE FROM messages_fts", [])
            .map_err(storage_error)?;
        let sql = format!(
            "INSERT INTO messages_fts (message_id, subject, addresses, body) SELECT m.id, m.subject, {ADDRESS_TEXT}, COALESCE(p.text_body, m.preview) FROM messages m LEFT JOIN parsed_cache p ON p.message_id = m.id"
        );
        self.sql().execute(&sql, []).map_err(storage_error)?;
        Ok(())
    }
}
