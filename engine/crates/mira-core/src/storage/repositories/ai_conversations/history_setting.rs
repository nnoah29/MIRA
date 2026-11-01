//! Interrupteur de l'historique des conversations d'IA, rangé dans `engine_settings` sous sa propre clé.

use mira_vocabulary::MiraError;
use rusqlite::{OptionalExtension, params};

use crate::storage::Tx;
use crate::storage::errors::storage_error;

const KEY: &str = "ai_history_enabled";

impl Tx<'_> {
    /// Dit si l'historique est activé ; un réglage jamais renseigné vaut activé
    pub fn ai_history_enabled(&self) -> Result<bool, MiraError> {
        let stored: Option<String> = self
            .sql()
            .query_row(
                "SELECT value FROM engine_settings WHERE key = ?1",
                [KEY],
                |row| row.get(0),
            )
            .optional()
            .map_err(storage_error)?;
        Ok(stored.is_none_or(|value| value != "0"))
    }

    /// Active ou coupe l'historique ; ne supprime aucune conversation
    pub fn set_ai_history_enabled(&self, enabled: bool) -> Result<(), MiraError> {
        let value = if enabled { "1" } else { "0" };
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
