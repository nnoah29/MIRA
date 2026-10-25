//! Écriture des réglages du moteur.

use mira_vocabulary::{EngineSettings, MiraError};
use rusqlite::params;

use super::read::{MUTED_ACCOUNTS, POLL_SECS, QUIET_HOURS};
use crate::storage::Tx;
use crate::storage::errors::storage_error;

impl Tx<'_> {
    /// Remplace les réglages : une valeur absente ou vide supprime la ligne
    pub fn put_engine_settings(&self, settings: &EngineSettings) -> Result<(), MiraError> {
        let poll = settings.poll_secs.map(|secs| secs.to_string());
        let quiet = settings
            .quiet_hours
            .map(|hours| format!("{}-{}", hours.from, hours.to));
        let muted = match settings.muted_accounts.is_empty() {
            true => None,
            false => Some(serde_json::to_string(&settings.muted_accounts).map_err(json_error)?),
        };
        self.put_engine_setting(POLL_SECS, poll.as_deref())?;
        self.put_engine_setting(MUTED_ACCOUNTS, muted.as_deref())?;
        self.put_engine_setting(QUIET_HOURS, quiet.as_deref())
    }

    /// Écrit la valeur d'un réglage, ou supprime sa ligne s'il n'en a pas
    fn put_engine_setting(&self, key: &str, value: Option<&str>) -> Result<(), MiraError> {
        let Some(value) = value else {
            return self.delete_engine_setting(key);
        };
        self.sql()
            .execute(
                "INSERT INTO engine_settings (key, value) VALUES (?1, ?2) \
                 ON CONFLICT (key) DO UPDATE SET value = excluded.value",
                params![key, value],
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

/// Erreur interne d'écriture JSON des comptes en sourdine
fn json_error(error: serde_json::Error) -> MiraError {
    MiraError::new(
        mira_vocabulary::ErrorCategory::Internal,
        mira_vocabulary::error::codes::internal::BUG,
        format!("réglages non sérialisables : {error}"),
    )
}
