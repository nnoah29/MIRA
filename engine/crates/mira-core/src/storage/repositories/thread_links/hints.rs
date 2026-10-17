//! Indices serveur de fil retrouvés dans les clés de fil existantes.

use mira_vocabulary::{AccountId, MiraError};

use crate::storage::Tx;
use crate::storage::errors::storage_error;

impl Tx<'_> {
    /// Indices serveur (identifiant de message, indice) lus dans les fils dont la clé commence par `hint:`
    pub fn thread_hints_of_messages(
        &self,
        account: &AccountId,
    ) -> Result<Vec<(String, String)>, MiraError> {
        let mut statement = self
            .sql()
            .prepare("SELECT m.id, substr(t.root_key, 6) FROM messages m JOIN threads t ON t.id = m.thread_id WHERE m.account_id = ?1 AND t.root_key LIKE 'hint:%'")
            .map_err(storage_error)?;
        let rows = statement
            .query_map([account.as_str()], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(storage_error)?;
        rows.map(|row| row.map_err(storage_error)).collect()
    }
}
