//! Exécution des requêtes qui rendent des résumés de messages.

use mira_vocabulary::{MailboxId, MessageId, MessageSummary, MiraError};

use super::rows::RawMessage;
use crate::storage::Tx;
use crate::storage::errors::storage_error;

impl Tx<'_> {
    /// Exécute une requête sélectionnant `COLUMNS` ; rend (date de tri, identifiant, résumé)
    pub(crate) fn query_summaries(
        &self,
        sql: &str,
        params: impl rusqlite::Params,
    ) -> Result<Vec<(i64, String, MessageSummary)>, MiraError> {
        let mut statement = self.sql().prepare(sql).map_err(storage_error)?;
        let raws = statement
            .query_map(params, RawMessage::from_row)
            .map_err(storage_error)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(storage_error)?;
        raws.into_iter()
            .map(|raw| {
                let (sort_date, id) = (raw.sort_date, raw.id.clone());
                let mailboxes = self.visible_mailboxes(&MessageId::from_raw(&id))?;
                Ok((sort_date, id, raw.into_summary(mailboxes)?))
            })
            .collect()
    }

    /// Boîtes où le message apparaît sans être marqué supprimé
    fn visible_mailboxes(&self, id: &MessageId) -> Result<Vec<MailboxId>, MiraError> {
        let mut statement = self
            .sql()
            .prepare("SELECT mailbox_id FROM appearances WHERE message_id = ?1 AND deleted = 0 ORDER BY mailbox_id")
            .map_err(storage_error)?;
        let rows = statement
            .query_map([id.as_str()], |row| row.get::<_, String>(0))
            .map_err(storage_error)?;
        rows.map(|row| row.map(MailboxId::from_raw).map_err(storage_error))
            .collect()
    }
}
