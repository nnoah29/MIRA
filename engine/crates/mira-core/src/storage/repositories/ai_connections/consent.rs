//! Consentement d'une connexion d'IA : posé une seule fois, remis à zéro quand la destination change.

use mira_vocabulary::{AiConnectionId, MiraError};
use rusqlite::params;

use super::errors::ensure_changed;
use crate::storage::Tx;
use crate::storage::errors::storage_error;

impl Tx<'_> {
    /// Note la date du consentement, une seule fois : une date déjà posée est gardée
    pub fn set_ai_consent(&self, id: &AiConnectionId, now: i64) -> Result<(), MiraError> {
        let changed = self
            .sql()
            .execute(
                "UPDATE ai_connections SET consent_at = COALESCE(consent_at, ?2) WHERE id = ?1",
                params![id.as_str(), now],
            )
            .map_err(storage_error)?;
        ensure_changed(changed)
    }

    /// Retire le consentement ; inconnue : `not_found.ai_connection`
    pub fn clear_ai_consent(&self, id: &AiConnectionId) -> Result<(), MiraError> {
        let changed = self
            .sql()
            .execute(
                "UPDATE ai_connections SET consent_at = NULL WHERE id = ?1",
                [id.as_str()],
            )
            .map_err(storage_error)?;
        ensure_changed(changed)
    }
}
