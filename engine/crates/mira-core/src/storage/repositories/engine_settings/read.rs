//! Lecture des réglages du moteur.

use mira_vocabulary::{EngineSettings, MiraError};
use rusqlite::{OptionalExtension, params};

use crate::storage::Tx;
use crate::storage::errors::storage_error;

/// Clé du relevé périodique
pub(super) const POLL_SECS: &str = "poll_secs";

impl Tx<'_> {
    /// Réglages enregistrés ; un réglage jamais renseigné reste absent
    pub fn get_engine_settings(&self) -> Result<EngineSettings, MiraError> {
        let stored: Option<String> = self
            .sql()
            .query_row(
                "SELECT value FROM engine_settings WHERE key = ?1",
                params![POLL_SECS],
                |row| row.get(0),
            )
            .optional()
            .map_err(storage_error)?;
        Ok(EngineSettings {
            poll_secs: stored.and_then(|text| text.parse().ok()),
        })
    }
}
