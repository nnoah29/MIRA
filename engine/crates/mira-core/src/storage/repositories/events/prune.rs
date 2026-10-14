//! Rétention bornée des événements.

use mira_vocabulary::{MiraError, Timestamp};

use crate::storage::Tx;
use crate::storage::errors::storage_error;

const MAX_AGE_MILLIS: i64 = 30 * 86_400_000;
const MAX_ROWS: u64 = 100_000;

impl Tx<'_> {
    /// Ne garde que les `keep` derniers événements et rend le nombre d'événements supprimés
    pub fn prune_events(&self, keep: u64) -> Result<u64, MiraError> {
        let keep = i64::try_from(keep).unwrap_or(i64::MAX);
        let removed = self
            .sql()
            .execute(
                "DELETE FROM events WHERE seq NOT IN (SELECT seq FROM events ORDER BY seq DESC LIMIT ?1)",
                [keep],
            )
            .map_err(storage_error)?;
        Ok(u64::try_from(removed).unwrap_or(0))
    }

    /// Supprime les événements antérieurs à l'instant donné et rend leur nombre
    pub fn prune_events_before(&self, cutoff: Timestamp) -> Result<u64, MiraError> {
        let removed = self
            .sql()
            .execute("DELETE FROM events WHERE at < ?1", [cutoff.as_millis()])
            .map_err(storage_error)?;
        Ok(u64::try_from(removed).unwrap_or(0))
    }

    /// Rétention des événements : au plus 30 jours et 100 000 lignes ; rend le nombre d'événements supprimés
    pub fn apply_event_retention(&self, now: Timestamp) -> Result<u64, MiraError> {
        let cutoff = Timestamp::from_millis(now.as_millis().saturating_sub(MAX_AGE_MILLIS));
        Ok(self.prune_events_before(cutoff)? + self.prune_events(MAX_ROWS)?)
    }
}
