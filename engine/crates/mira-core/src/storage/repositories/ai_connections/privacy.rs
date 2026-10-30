//! Confidentialité d'une connexion d'IA : types de contenu autorisés et réglage du masquage.

use mira_vocabulary::{AiConnectionId, ContentKind, MaskingSettings, MiraError};
use rusqlite::params;

use super::errors::ensure_changed;
use crate::storage::Tx;
use crate::storage::errors::storage_error;
use crate::storage::json_columns::to_json;

impl Tx<'_> {
    /// Remplace la liste des types de contenu autorisés ; inconnue : `not_found.ai_connection`
    pub fn set_ai_allowed_kinds(
        &self,
        id: &AiConnectionId,
        kinds: &[ContentKind],
    ) -> Result<(), MiraError> {
        let changed = self
            .sql()
            .execute(
                "UPDATE ai_connections SET allowed_kinds = ?2 WHERE id = ?1",
                params![id.as_str(), to_json(&kinds)?],
            )
            .map_err(storage_error)?;
        ensure_changed(changed)
    }

    /// Remplace le réglage du masquage ; inconnue : `not_found.ai_connection`
    pub fn set_ai_masking(
        &self,
        id: &AiConnectionId,
        masking: &MaskingSettings,
    ) -> Result<(), MiraError> {
        let changed = self
            .sql()
            .execute(
                "UPDATE ai_connections SET masking_json = ?2 WHERE id = ?1",
                params![id.as_str(), to_json(masking)?],
            )
            .map_err(storage_error)?;
        ensure_changed(changed)
    }
}
