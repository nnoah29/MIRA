//! Lectures du miroir utiles aux règles : messages candidats d'un compte, fil déjà existant.

use mira_vocabulary::{AccountId, MessageId, MessageSummary, MiraError, ThreadId};
use rusqlite::params;

use crate::storage::Tx;
use crate::storage::errors::storage_error;
use crate::storage::repositories::messages::MESSAGE_COLUMNS;

/// Rôles des boîtes jamais évaluées par les règles : Envoyés, Brouillons, Corbeille, Spam.
const EXCLUDED_ROLES: &str = "('sent', 'drafts', 'trash', 'junk')";

impl Tx<'_> {
    /// Messages les plus récents du compte visibles dans une boîte évaluée, avec leur date de tri
    pub fn rule_candidates(
        &self,
        account: &AccountId,
        limit: u32,
    ) -> Result<Vec<(i64, MessageSummary)>, MiraError> {
        let sql = format!(
            "SELECT {MESSAGE_COLUMNS} FROM messages m WHERE m.account_id = ?1 AND EXISTS (SELECT 1 FROM appearances a JOIN mailboxes b ON b.id = a.mailbox_id WHERE a.message_id = m.id AND a.deleted = 0 AND b.role NOT IN {EXCLUDED_ROLES}) ORDER BY m.sort_date DESC, m.id DESC LIMIT ?2"
        );
        let rows = self.query_summaries(&sql, params![account.as_str(), i64::from(limit)])?;
        Ok(rows
            .into_iter()
            .map(|(sort_date, _, summary)| (sort_date, summary))
            .collect())
    }

    /// Vrai si le fil contient déjà un autre message que celui-ci
    pub fn thread_has_other_messages(
        &self,
        thread: &ThreadId,
        message: &MessageId,
    ) -> Result<bool, MiraError> {
        self.sql()
            .query_row(
                "SELECT EXISTS (SELECT 1 FROM messages WHERE thread_id = ?1 AND id <> ?2)",
                params![thread.as_str(), message.as_str()],
                |row| row.get(0),
            )
            .map_err(storage_error)
    }
}
