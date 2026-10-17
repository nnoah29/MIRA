//! Correspondance des identifiants serveur et recoupement de dernier recours (D-13, P7).

use mira_vocabulary::{AccountId, MailboxId, MessageId, MiraError, Timestamp};
use rusqlite::{OptionalExtension, params};

use super::kind::ServerIdKind;
use crate::storage::Tx;
use crate::storage::errors::storage_error;

impl Tx<'_> {
    /// Associe un identifiant serveur à un message (remplace une association existante)
    pub fn map_server_id(
        &self,
        account: &AccountId,
        kind: ServerIdKind,
        value: &str,
        message: &MessageId,
    ) -> Result<(), MiraError> {
        self.sql()
            .execute(
                "INSERT INTO server_ids (account_id, kind, value, message_id) VALUES (?1, ?2, ?3, ?4) ON CONFLICT (account_id, kind, value) DO UPDATE SET message_id = excluded.message_id",
                params![account.as_str(), kind.as_str(), value, message.as_str()],
            )
            .map_err(storage_error)?;
        Ok(())
    }

    /// Retrouve le message associé à un identifiant serveur
    pub fn find_by_server_id(
        &self,
        account: &AccountId,
        kind: ServerIdKind,
        value: &str,
    ) -> Result<Option<MessageId>, MiraError> {
        self.sql()
            .query_row(
                "SELECT message_id FROM server_ids WHERE account_id = ?1 AND kind = ?2 AND value = ?3",
                params![account.as_str(), kind.as_str(), value],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .map(|found| found.map(MessageId::from_raw))
            .map_err(storage_error)
    }

    /// Oublie les identifiants dérivés d'UID d'une boîte (valeurs « boîte/validité/uid »)
    pub fn forget_uid_ids_for_mailbox(&self, mailbox: &MailboxId) -> Result<(), MiraError> {
        self.sql()
            .execute(
                "DELETE FROM server_ids WHERE kind = 'uid' AND substr(value, 1, length(?1) + 1) = ?1 || '/'",
                [mailbox.as_str()],
            )
            .map_err(storage_error)?;
        Ok(())
    }

    /// Candidats de dernier recours (même Message-ID, date et taille) ; l'appelant tranche
    pub fn find_by_fingerprint(
        &self,
        account: &AccountId,
        message_id_header: &str,
        date: Option<Timestamp>,
        size: u64,
    ) -> Result<Vec<MessageId>, MiraError> {
        let mut statement = self
            .sql()
            .prepare("SELECT id FROM messages WHERE account_id = ?1 AND message_id_header = ?2 AND date IS ?3 AND size = ?4 ORDER BY id")
            .map_err(storage_error)?;
        let size = i64::try_from(size).unwrap_or(i64::MAX);
        let args = params![
            account.as_str(),
            message_id_header,
            date.map(|d| d.as_millis()),
            size
        ];
        let rows = statement
            .query_map(args, |row| row.get::<_, String>(0))
            .map_err(storage_error)?;
        rows.map(|row| row.map(MessageId::from_raw).map_err(storage_error))
            .collect()
    }
}
