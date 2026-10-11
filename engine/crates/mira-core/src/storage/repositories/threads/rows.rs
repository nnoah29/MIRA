//! Conversion entre une ligne agrégée de fil et un `Thread`.

use mira_vocabulary::{AccountId, AddressEntry, Thread, ThreadId, Timestamp};

pub(super) const COLUMNS: &str =
    "t.id, t.account_id, t.subject, COUNT(m.id), COALESCE(SUM(m.seen = 0), 0), MAX(m.sort_date)";

/// Ligne agrégée d'un fil : identité, sujet, nombres de messages et date du dernier.
pub(super) struct RawThread {
    id: String,
    account: String,
    subject: String,
    counts: (i64, i64),
    last: Option<i64>,
}

impl RawThread {
    /// Lit une ligne sélectionnée avec `COLUMNS`
    pub(super) fn from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get(0)?,
            account: row.get(1)?,
            subject: row.get(2)?,
            counts: (row.get(3)?, row.get(4)?),
            last: row.get(5)?,
        })
    }

    /// Date de tri du dernier message du fil (0 si le fil est vide)
    pub(super) fn last_millis(&self) -> i64 {
        self.last.unwrap_or(0)
    }

    /// Identifiant du fil, sous forme de texte
    pub(super) fn thread_id(&self) -> String {
        self.id.clone()
    }

    /// Reconstruit le fil du domaine avec ses participants
    pub(super) fn into_thread(self, participants: Vec<AddressEntry>) -> Thread {
        Thread {
            id: ThreadId::from_raw(self.id),
            account: AccountId::from_raw(self.account),
            subject: self.subject,
            message_count: u32::try_from(self.counts.0).unwrap_or(u32::MAX),
            unread_count: u32::try_from(self.counts.1).unwrap_or(u32::MAX),
            last_date: self.last.map(Timestamp::from_millis),
            participants,
        }
    }
}
