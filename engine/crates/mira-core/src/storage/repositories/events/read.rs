//! Lecture des événements à partir d'un numéro.

use mira_vocabulary::{Event, EventSeq, MiraError};

use super::rows::{COLUMNS, RawEvent};
use crate::storage::Tx;
use crate::storage::errors::storage_error;

impl Tx<'_> {
    /// Relit au plus `limit` événements de numéro supérieur à `after`, dans l'ordre
    pub fn events_after(
        &self,
        after: Option<EventSeq>,
        limit: u32,
    ) -> Result<Vec<Event>, MiraError> {
        let start = after.map_or(0, |seq| i64::try_from(seq.0).unwrap_or(i64::MAX));
        let mut statement = self
            .sql()
            .prepare(&format!(
                "SELECT {COLUMNS} FROM events WHERE seq > ?1 ORDER BY seq LIMIT ?2"
            ))
            .map_err(storage_error)?;
        let raws = statement
            .query_map([start, i64::from(limit)], RawEvent::from_row)
            .map_err(storage_error)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(storage_error)?;
        raws.into_iter().map(RawEvent::into_event).collect()
    }
}
