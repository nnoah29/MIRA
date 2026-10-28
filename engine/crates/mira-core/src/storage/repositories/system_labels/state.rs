//! État d'un libellé système sur une conversation, rattaché à (compte, clé du fil).

use mira_vocabulary::{AccountId, MiraError, SystemLabelAssignment, SystemLabelKind, ThreadId};
use rusqlite::{OptionalExtension, params};

use crate::storage::Tx;
use crate::storage::errors::storage_error;

/// État enregistré : posé ou retiré, par la détection ou par l'utilisateur, avec la date du dernier message alors connu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SystemState {
    pub on: bool,
    pub manual: bool,
    pub as_of: i64,
}

impl Tx<'_> {
    /// Relit l'état d'une sorte sur une conversation, ou rien s'il n'a jamais été écrit
    pub fn system_label_state(
        &self,
        account: &AccountId,
        root_key: &str,
        kind: SystemLabelKind,
    ) -> Result<Option<SystemState>, MiraError> {
        self.sql()
            .query_row(
                "SELECT on_state, source, as_of FROM thread_system_labels \
                 WHERE account_id = ?1 AND root_key = ?2 AND kind = ?3",
                params![account.as_str(), root_key, kind.key()],
                |row| {
                    Ok(SystemState {
                        on: row.get::<_, i64>(0)? == 1,
                        manual: row.get::<_, String>(1)? == "manual",
                        as_of: row.get(2)?,
                    })
                },
            )
            .optional()
            .map_err(storage_error)
    }

    /// Écrit l'état d'une sorte sur une conversation, en remplaçant l'ancien
    pub fn put_system_label_state(
        &self,
        account: &AccountId,
        root_key: &str,
        kind: SystemLabelKind,
        state: SystemState,
    ) -> Result<(), MiraError> {
        let source = if state.manual { "manual" } else { "auto" };
        self.sql()
            .execute(
                "INSERT OR REPLACE INTO thread_system_labels (account_id, root_key, kind, on_state, source, as_of) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![account.as_str(), root_key, kind.key(), i64::from(state.on), source, state.as_of],
            )
            .map_err(storage_error)?;
        Ok(())
    }

    /// Efface l'état d'une sorte sur une conversation
    pub fn delete_system_label_state(
        &self,
        account: &AccountId,
        root_key: &str,
        kind: SystemLabelKind,
    ) -> Result<(), MiraError> {
        self.sql()
            .execute(
                "DELETE FROM thread_system_labels WHERE account_id = ?1 AND root_key = ?2 AND kind = ?3",
                params![account.as_str(), root_key, kind.key()],
            )
            .map_err(storage_error)?;
        Ok(())
    }

    /// Libellés système posés sur chaque conversation qui existe encore, groupés par conversation
    pub fn list_system_label_assignments(&self) -> Result<Vec<SystemLabelAssignment>, MiraError> {
        let mut statement = self
            .sql()
            .prepare(
                "SELECT t.id, s.kind FROM thread_system_labels s \
                 JOIN threads t ON t.account_id = s.account_id AND t.root_key = s.root_key \
                 WHERE s.on_state = 1 ORDER BY t.id, s.kind",
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

/// Regroupe les couples (conversation, clé de sorte) triés par conversation ; une clé inconnue est ignorée
fn group_by_thread(rows: Vec<(String, String)>) -> Vec<SystemLabelAssignment> {
    let mut grouped: Vec<SystemLabelAssignment> = Vec::new();
    for (thread, key) in rows {
        let Some(kind) = SystemLabelKind::from_key(&key) else {
            continue;
        };
        match grouped.last_mut() {
            Some(last) if last.thread.as_str() == thread => last.kinds.push(kind),
            _ => grouped.push(SystemLabelAssignment {
                thread: ThreadId::from_raw(thread),
                kinds: vec![kind],
            }),
        }
    }
    grouped
}
