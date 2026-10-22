//! Conversion entre une ligne de la table `events` et un `Event`.

use mira_vocabulary::{Event, EventCategory, EventSeq, MiraError, Timestamp};

use crate::storage::json_columns::from_json;

pub(super) const COLUMNS: &str = "seq, at, kind_json";

/// Ligne brute de la table `events`, dans l'ordre de `COLUMNS`.
pub(super) struct RawEvent {
    seq: i64,
    at: i64,
    kind_json: String,
}

impl RawEvent {
    /// Lit une ligne sélectionnée avec `COLUMNS`
    pub(super) fn from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Self> {
        Ok(Self {
            seq: row.get(0)?,
            at: row.get(1)?,
            kind_json: row.get(2)?,
        })
    }

    /// Reconstruit l'événement du domaine depuis la ligne
    pub(super) fn into_event(self) -> Result<Event, MiraError> {
        Ok(Event {
            seq: EventSeq(u64::try_from(self.seq).unwrap_or(0)),
            at: Timestamp::from_millis(self.at),
            kind: from_json(&self.kind_json)?,
        })
    }
}

/// Donne le nom stocké d'une catégorie d'événement
pub(super) fn category_name(category: EventCategory) -> &'static str {
    match category {
        EventCategory::Engine => "engine",
        EventCategory::Account => "account",
        EventCategory::Sync => "sync",
        EventCategory::Mailbox => "mailbox",
        EventCategory::Message => "message",
        EventCategory::Thread => "thread",
        EventCategory::Action => "action",
        EventCategory::Send => "send",
        EventCategory::Alert => "alert",
        EventCategory::Log => "log",
        EventCategory::Rule => "rule",
        EventCategory::LocalLabel => "local_label",
        EventCategory::Label => "label",
    }
}
