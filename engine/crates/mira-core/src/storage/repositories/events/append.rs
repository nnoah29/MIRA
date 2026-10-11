//! Ajout d'un événement numéroté.

use mira_vocabulary::{Event, EventKind, EventSeq, MiraError, Timestamp};
use rusqlite::params;

use super::rows::category_name;
use crate::storage::Tx;
use crate::storage::errors::storage_error;
use crate::storage::json_columns::to_json;

impl Tx<'_> {
    /// Enregistre un événement et rend son numéro, strictement croissant et jamais réutilisé
    pub fn append_event(&self, at: Timestamp, kind: &EventKind) -> Result<Event, MiraError> {
        self.sql()
            .execute(
                "INSERT INTO events (at, account_id, category, kind_json) VALUES (?1, ?2, ?3, ?4)",
                params![
                    at.as_millis(),
                    kind.account().map(|account| account.as_str()),
                    category_name(kind.category()),
                    to_json(kind)?,
                ],
            )
            .map_err(storage_error)?;
        let seq = self.sql().last_insert_rowid();
        Ok(Event {
            seq: EventSeq(u64::try_from(seq).unwrap_or(0)),
            at,
            kind: kind.clone(),
        })
    }
}
