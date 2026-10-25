//! Lectures sur les modèles de messages.

use mira_vocabulary::{MessageTemplate, MiraError, TemplateId};
use rusqlite::OptionalExtension;

use super::errors::template_not_found;
use super::rows::{COLUMNS, template_from_row};
use crate::storage::Tx;
use crate::storage::errors::storage_error;

impl Tx<'_> {
    /// Relit un modèle, ou rien s'il n'existe pas
    pub fn get_template(&self, id: &TemplateId) -> Result<Option<MessageTemplate>, MiraError> {
        self.sql()
            .query_row(
                &format!("SELECT {COLUMNS} FROM message_templates WHERE id = ?1"),
                [id.as_str()],
                template_from_row,
            )
            .optional()
            .map_err(storage_error)
    }

    /// Liste les modèles par nom (sans tenir compte de la casse), puis identifiant
    pub fn list_templates(&self) -> Result<Vec<MessageTemplate>, MiraError> {
        let mut statement = self
            .sql()
            .prepare(&format!(
                "SELECT {COLUMNS} FROM message_templates ORDER BY name_key, id"
            ))
            .map_err(storage_error)?;
        statement
            .query_map([], template_from_row)
            .map_err(storage_error)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(storage_error)
    }

    /// Nombre de modèles enregistrés
    pub fn count_templates(&self) -> Result<usize, MiraError> {
        self.sql()
            .query_row("SELECT COUNT(*) FROM message_templates", [], |row| {
                row.get::<_, i64>(0)
            })
            .map(|count| usize::try_from(count).unwrap_or(0))
            .map_err(storage_error)
    }
}

/// Relit un modèle ou échoue avec `not_found.template`
pub fn require_template(tx: &Tx<'_>, id: &TemplateId) -> Result<MessageTemplate, MiraError> {
    tx.get_template(id)?.ok_or_else(template_not_found)
}
