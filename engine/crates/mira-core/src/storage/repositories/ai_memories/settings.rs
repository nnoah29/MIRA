//! Réglages de la mémoire de l'IA, rangés en JSON dans `engine_settings` sous leur propre clé.

use mira_vocabulary::{MemorySettings, MiraError};
use rusqlite::{OptionalExtension, params};

use crate::storage::Tx;
use crate::storage::errors::storage_error;

const KEY: &str = "ai_memory";

impl Tx<'_> {
    /// Rend les réglages ; jamais renseignés ou illisibles : mémoire désactivée, calcul sur l'appareil
    pub fn ai_memory_settings(&self) -> Result<MemorySettings, MiraError> {
        let stored: Option<String> = self
            .sql()
            .query_row(
                "SELECT value FROM engine_settings WHERE key = ?1",
                [KEY],
                |row| row.get(0),
            )
            .optional()
            .map_err(storage_error)?;
        Ok(stored
            .and_then(|text| serde_json::from_str(&text).ok())
            .unwrap_or_default())
    }

    /// Enregistre les réglages
    pub fn set_ai_memory_settings(&self, settings: &MemorySettings) -> Result<(), MiraError> {
        let value = serde_json::to_string(settings).unwrap_or_default();
        self.sql()
            .execute(
                "INSERT INTO engine_settings (key, value) VALUES (?1, ?2) \
                 ON CONFLICT (key) DO UPDATE SET value = excluded.value",
                params![KEY, value],
            )
            .map_err(storage_error)?;
        Ok(())
    }
}
