//! Lectures sur les boîtes.

use mira_vocabulary::{AccountId, Mailbox, MailboxId, MailboxRole, MiraError};
use rusqlite::OptionalExtension;

use super::role::role_name;
use super::rows::{COLUMNS, RawMailbox};
use crate::storage::Tx;
use crate::storage::errors::storage_error;

impl Tx<'_> {
    /// Relit une boîte, ou rien si elle n'existe pas
    pub fn get_mailbox(&self, id: &MailboxId) -> Result<Option<Mailbox>, MiraError> {
        let sql = format!("SELECT {COLUMNS} FROM mailboxes WHERE id = ?1");
        self.query_one_mailbox(&sql, [id.as_str()])
    }

    /// Liste les boîtes d'un compte, par nom
    pub fn list_mailboxes(&self, account: &AccountId) -> Result<Vec<Mailbox>, MiraError> {
        let sql = format!("SELECT {COLUMNS} FROM mailboxes WHERE account_id = ?1 ORDER BY name");
        let mut statement = self.sql().prepare(&sql).map_err(storage_error)?;
        let raws = statement
            .query_map([account.as_str()], RawMailbox::from_row)
            .map_err(storage_error)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(storage_error)?;
        raws.into_iter().map(RawMailbox::into_mailbox).collect()
    }

    /// Trouve la première boîte d'un compte portant un rôle
    pub fn find_mailbox_by_role(
        &self,
        account: &AccountId,
        role: MailboxRole,
    ) -> Result<Option<Mailbox>, MiraError> {
        let sql = format!(
            "SELECT {COLUMNS} FROM mailboxes WHERE account_id = ?1 AND role = ?2 ORDER BY name LIMIT 1"
        );
        self.query_one_mailbox(&sql, [account.as_str(), role_name(role)])
    }

    /// Trouve la boîte d'un compte portant un nom serveur
    pub fn find_mailbox_by_name(
        &self,
        account: &AccountId,
        name: &str,
    ) -> Result<Option<Mailbox>, MiraError> {
        let sql = format!("SELECT {COLUMNS} FROM mailboxes WHERE account_id = ?1 AND name = ?2");
        self.query_one_mailbox(&sql, [account.as_str(), name])
    }

    /// Exécute une requête qui rend au plus une boîte
    fn query_one_mailbox(
        &self,
        sql: &str,
        params: impl rusqlite::Params,
    ) -> Result<Option<Mailbox>, MiraError> {
        let raw = self
            .sql()
            .query_row(sql, params, RawMailbox::from_row)
            .optional()
            .map_err(storage_error)?;
        raw.map(RawMailbox::into_mailbox).transpose()
    }
}
