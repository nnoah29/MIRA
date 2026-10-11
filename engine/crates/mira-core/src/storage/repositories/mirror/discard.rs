//! Abandon du miroir d'un compte.

use mira_vocabulary::{AccountId, MiraError};

use crate::storage::Tx;
use crate::storage::errors::storage_error;

const STATEMENTS: [&str; 5] = [
    "DELETE FROM messages_fts WHERE message_id IN (SELECT id FROM messages WHERE account_id = ?1)",
    "DELETE FROM server_ids WHERE account_id = ?1",
    "DELETE FROM messages WHERE account_id = ?1",
    "DELETE FROM threads WHERE account_id = ?1",
    "DELETE FROM mailboxes WHERE account_id = ?1",
];

impl Tx<'_> {
    /// Jette le miroir d'un compte (boîtes, messages et dépendances, fils, index) ; garde comptes, identités, brouillons et actions
    pub fn discard_account_mirror(&self, account: &AccountId) -> Result<(), MiraError> {
        for statement in STATEMENTS {
            self.sql()
                .execute(statement, [account.as_str()])
                .map_err(storage_error)?;
        }
        Ok(())
    }
}
