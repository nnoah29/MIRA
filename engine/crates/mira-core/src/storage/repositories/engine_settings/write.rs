//! Écriture des réglages du moteur.

use mira_vocabulary::{EngineSettings, MiraError};
use rusqlite::params;

use super::read::POLL_SECS;
use crate::storage::Tx;
use crate::storage::errors::storage_error;

impl Tx<'_> {
    /// Remplace les réglages : une valeur absente supprime la ligne
    pub fn put_engine_settings(&self, settings: &EngineSettings) -> Result<(), MiraError> {
        let Some(secs) = settings.poll_secs else {
            return self.delete_engine_setting(POLL_SECS);
        };
        self.sql()
            .execute(
                "INSERT INTO engine_settings (key, value) VALUES (?1, ?2) \
                 ON CONFLICT (key) DO UPDATE SET value = excluded.value",
                params![POLL_SECS, secs.to_string()],
            )
            .map_err(storage_error)?;
        Ok(())
    }

    /// Supprime la ligne d'un réglage (sans effet s'il n'est pas renseigné)
    fn delete_engine_setting(&self, key: &str) -> Result<(), MiraError> {
        self.sql()
            .execute("DELETE FROM engine_settings WHERE key = ?1", params![key])
            .map_err(storage_error)?;
        Ok(())
    }
}
