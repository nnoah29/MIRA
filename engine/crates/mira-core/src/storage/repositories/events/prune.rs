//! Rétention bornée des événements.

use mira_vocabulary::MiraError;

use crate::storage::Tx;
use crate::storage::errors::storage_error;

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
}
