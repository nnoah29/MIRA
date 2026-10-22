//! Poses de libellés locaux sur les conversations, rattachées à (compte, clé du fil).

use mira_vocabulary::{AccountId, LocalLabelAssignment, LocalLabelId, MiraError, ThreadId};
use rusqlite::{OptionalExtension, params};

use super::rows::{label_not_found, thread_not_found};
use crate::storage::Tx;
use crate::storage::errors::storage_error;

impl Tx<'_> {
    /// Pose ou retire un libellé sur un fil ; libellé ou fil inconnu : refusé sans effet
    pub fn set_label_on_thread(
        &self,
        label: &LocalLabelId,
        thread: &ThreadId,
        on: bool,
    ) -> Result<(), MiraError> {
        if self.get_local_label(label)?.is_none() {
            return Err(label_not_found());
        }
        let (account, key) = self.thread_root_key(thread)?.ok_or_else(thread_not_found)?;
        let sql = if on {
            "INSERT OR IGNORE INTO thread_local_labels (label_id, account_id, root_key) VALUES (?1, ?2, ?3)"
        } else {
            "DELETE FROM thread_local_labels WHERE label_id = ?1 AND account_id = ?2 AND root_key = ?3"
        };
        self.sql()
            .execute(sql, params![label.as_str(), account.as_str(), key])
            .map_err(storage_error)?;
        Ok(())
    }

    /// Compte et clé de racine d'un fil, ou rien s'il n'existe pas
    pub fn thread_root_key(
        &self,
        thread: &ThreadId,
    ) -> Result<Option<(AccountId, String)>, MiraError> {
        self.sql()
            .query_row(
                "SELECT account_id, root_key FROM threads WHERE id = ?1",
                [thread.as_str()],
                |row| Ok((AccountId::from_raw(row.get::<_, String>(0)?), row.get(1)?)),
            )
            .optional()
            .map_err(storage_error)
    }

    /// Libellés posés sur un fil, par identifiant
    pub fn labels_of_thread(&self, thread: &ThreadId) -> Result<Vec<LocalLabelId>, MiraError> {
        let mut statement = self
            .sql()
            .prepare(
                "SELECT tl.label_id FROM thread_local_labels tl \
                 JOIN threads t ON t.account_id = tl.account_id AND t.root_key = tl.root_key \
                 WHERE t.id = ?1 ORDER BY tl.label_id",
            )
            .map_err(storage_error)?;
        statement
            .query_map([thread.as_str()], |row| {
                Ok(LocalLabelId::from_raw(row.get::<_, String>(0)?))
            })
            .map_err(storage_error)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(storage_error)
    }

    /// Poses de tous les fils qui existent encore, groupées par fil
    pub fn list_label_assignments(&self) -> Result<Vec<LocalLabelAssignment>, MiraError> {
        let mut statement = self
            .sql()
            .prepare(
                "SELECT t.id, tl.label_id FROM thread_local_labels tl \
                 JOIN threads t ON t.account_id = tl.account_id AND t.root_key = tl.root_key \
                 ORDER BY t.id, tl.label_id",
            )
            .map_err(storage_error)?;
        let rows = statement
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(storage_error)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(storage_error)?;
        Ok(group_by_thread(rows))
    }
}

/// Regroupe les couples (fil, libellé) triés par fil
fn group_by_thread(rows: Vec<(String, String)>) -> Vec<LocalLabelAssignment> {
    let mut grouped: Vec<LocalLabelAssignment> = Vec::new();
    for (thread, label) in rows {
        let label = LocalLabelId::from_raw(label);
        match grouped.last_mut() {
            Some(last) if last.thread.as_str() == thread => last.labels.push(label),
            _ => grouped.push(LocalLabelAssignment {
                thread: ThreadId::from_raw(thread),
                labels: vec![label],
            }),
        }
    }
    grouped
}
