//! Recherches servant au rattachement d'un message à un fil.

use mira_vocabulary::{AccountId, MessageId, MiraError, ThreadId};
use rusqlite::{OptionalExtension, params};

use crate::storage::Tx;
use crate::storage::errors::storage_error;

impl Tx<'_> {
    /// Fil d'un autre message du compte portant cet en-tête `Message-ID`, s'il en a un
    pub fn thread_of_known_header(
        &self,
        account: &AccountId,
        header: &str,
        except: &MessageId,
    ) -> Result<Option<ThreadId>, MiraError> {
        let found = self
            .sql()
            .query_row(
                "SELECT thread_id FROM messages WHERE account_id = ?1 AND message_id_header = ?2 AND id <> ?3 AND thread_id IS NOT NULL ORDER BY sort_date, id LIMIT 1",
                params![account.as_str(), header, except.as_str()],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .map_err(storage_error)?;
        Ok(found.map(ThreadId::from_raw))
    }

    /// Couples (clé de racine, fil) existants d'un compte, pour conserver les identifiants au recalcul
    pub fn thread_ids_by_root_key(
        &self,
        account: &AccountId,
    ) -> Result<Vec<(String, ThreadId)>, MiraError> {
        let mut statement = self
            .sql()
            .prepare("SELECT root_key, id FROM threads WHERE account_id = ?1")
            .map_err(storage_error)?;
        let rows = statement
            .query_map([account.as_str()], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(storage_error)?;
        rows.map(|row| {
            let (key, id) = row.map_err(storage_error)?;
            Ok((key, ThreadId::from_raw(id)))
        })
        .collect()
    }
}
