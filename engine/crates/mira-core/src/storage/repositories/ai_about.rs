//! Texte « ce que tu dois savoir de moi », écrit par l'utilisateur et rangé tel quel dans `engine_settings`.

use mira_vocabulary::error::codes::invalid_input::AI_ABOUT_TEXT;
use mira_vocabulary::{ErrorCategory, MiraError};
use rusqlite::{OptionalExtension, params};

use crate::storage::Tx;
use crate::storage::errors::storage_error;

const KEY: &str = "ai_about";

/// Taille maximale du texte, en caractères.
pub const ABOUT_MAX_CHARS: usize = 1_000;

/// Texte rogné, fins de ligne Windows normalisées ; trop long : `invalid_input.ai_about_text`.
fn checked(text: &str) -> Result<String, MiraError> {
    let text = text.replace("\r\n", "\n").trim().to_string();
    match text.chars().count() > ABOUT_MAX_CHARS {
        true => Err(MiraError::new(
            ErrorCategory::InvalidInput,
            AI_ABOUT_TEXT,
            "le texte est trop long (1 000 caractères au plus)",
        )),
        false => Ok(text),
    }
}

impl Tx<'_> {
    /// Texte enregistré ; vide tant que l'utilisateur n'a rien écrit
    pub fn ai_about(&self) -> Result<String, MiraError> {
        let stored: Option<String> = self
            .sql()
            .query_row(
                "SELECT value FROM engine_settings WHERE key = ?1",
                [KEY],
                |row| row.get(0),
            )
            .optional()
            .map_err(storage_error)?;
        Ok(stored.unwrap_or_default())
    }

    /// Enregistre le texte et rend ce qui a été gardé ; un texte vide efface le champ
    pub fn set_ai_about(&self, text: &str) -> Result<String, MiraError> {
        let kept = checked(text)?;
        self.sql()
            .execute(
                "INSERT INTO engine_settings (key, value) VALUES (?1, ?2) \
                 ON CONFLICT (key) DO UPDATE SET value = excluded.value",
                params![KEY, kept],
            )
            .map_err(storage_error)?;
        Ok(kept)
    }
}
