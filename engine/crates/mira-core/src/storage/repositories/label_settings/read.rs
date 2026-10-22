//! Lectures sur les réglages des libellés.

use mira_vocabulary::{LabelRef, LabelSettings, MiraError};
use rusqlite::params;

use super::rows::{COLUMNS, label_of, settings_from_row};
use crate::storage::Tx;
use crate::storage::errors::storage_error;

impl Tx<'_> {
    /// Réglages d'un libellé, ou les réglages par défaut s'il n'en a pas
    pub fn get_label_settings(&self, label: &LabelRef) -> Result<LabelSettings, MiraError> {
        let found = self.sql().query_row(
            &format!("SELECT {COLUMNS} FROM label_settings WHERE kind = ?1 AND label_id = ?2"),
            params![label.kind.as_str(), label.id],
            |row| settings_from_row(row, 0),
        );
        match found {
            Ok(settings) => Ok(settings),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(LabelSettings::default()),
            Err(error) => Err(storage_error(error)),
        }
    }

    /// Tous les réglages enregistrés, avec le libellé qu'ils concernent
    pub fn list_label_settings(&self) -> Result<Vec<(LabelRef, LabelSettings)>, MiraError> {
        let mut statement = self
            .sql()
            .prepare(&format!(
                "SELECT kind, label_id, {COLUMNS} FROM label_settings"
            ))
            .map_err(storage_error)?;
        let rows = statement
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    settings_from_row(row, 2)?,
                ))
            })
            .map_err(storage_error)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(storage_error)?;
        Ok(rows
            .into_iter()
            .filter_map(|(kind, id, settings)| Some((label_of(&kind, id)?, settings)))
            .collect())
    }
}
