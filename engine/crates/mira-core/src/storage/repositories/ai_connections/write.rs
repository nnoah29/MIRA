//! Écritures sur les connexions d'IA : création, mise à jour, suppression, défaut, modèles.

use mira_vocabulary::{AiConnection, AiConnectionId, AiModelList, MiraError};
use rusqlite::params;

use super::errors::ensure_changed;
use super::rows::models_to_columns;
use crate::storage::Tx;
use crate::storage::errors::storage_error;
use crate::storage::json_columns::to_json;

impl Tx<'_> {
    /// Insère une connexion ; un second défaut est refusé par l'index unique
    pub fn insert_ai_connection(
        &self,
        connection: &AiConnection,
        now: i64,
    ) -> Result<(), MiraError> {
        let (models_json, fetched_at) = models_to_columns(connection.models.as_ref())?;
        self.sql()
            .execute(
                "INSERT INTO ai_connections (id, provider_id, label, base_url, default_model, is_default, \
                 consent_at, models_json, models_fetched_at, created_at, updated_at) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?10)",
                params![
                    connection.id.as_str(),
                    connection.provider.0,
                    connection.label,
                    connection.base_url,
                    connection.default_model,
                    connection.is_default,
                    connection.consent_at.map(|at| at.as_millis()),
                    models_json,
                    fetched_at,
                    now
                ],
            )
            .map_err(storage_error)?;
        Ok(())
    }

    /// Remplace libellé, adresse et modèle par défaut, sans toucher consentement, défaut ni modèles gardés
    pub fn update_ai_connection(
        &self,
        id: &AiConnectionId,
        label: &str,
        base_url: Option<&str>,
        default_model: Option<&str>,
        now: i64,
    ) -> Result<(), MiraError> {
        let changed = self
            .sql()
            .execute(
                "UPDATE ai_connections SET label = ?2, base_url = ?3, default_model = ?4, updated_at = ?5 \
                 WHERE id = ?1",
                params![id.as_str(), label, base_url, default_model, now],
            )
            .map_err(storage_error)?;
        ensure_changed(changed)
    }

    /// Supprime une connexion ; inconnue : `not_found.ai_connection`
    pub fn delete_ai_connection(&self, id: &AiConnectionId) -> Result<(), MiraError> {
        let changed = self
            .sql()
            .execute("DELETE FROM ai_connections WHERE id = ?1", [id.as_str()])
            .map_err(storage_error)?;
        ensure_changed(changed)
    }

    /// Désigne la connexion par défaut en retirant l'ancienne, dans la même transaction
    pub fn set_default_ai_connection(&self, id: &AiConnectionId) -> Result<(), MiraError> {
        self.require_ai_connection(id)?;
        self.sql()
            .execute(
                "UPDATE ai_connections SET is_default = 0 WHERE is_default = 1 AND id <> ?1",
                [id.as_str()],
            )
            .map_err(storage_error)?;
        self.sql()
            .execute(
                "UPDATE ai_connections SET is_default = 1 WHERE id = ?1",
                [id.as_str()],
            )
            .map_err(storage_error)?;
        Ok(())
    }

    /// Garde la liste de modèles relevée et sa date ; inconnue : `not_found.ai_connection`
    pub fn save_ai_models(&self, id: &AiConnectionId, list: &AiModelList) -> Result<(), MiraError> {
        let changed = self
            .sql()
            .execute(
                "UPDATE ai_connections SET models_json = ?2, models_fetched_at = ?3 WHERE id = ?1",
                params![
                    id.as_str(),
                    to_json(&list.models)?,
                    list.fetched_at.as_millis()
                ],
            )
            .map_err(storage_error)?;
        ensure_changed(changed)
    }
}
