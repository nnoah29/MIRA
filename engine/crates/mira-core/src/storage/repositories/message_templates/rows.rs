//! Lignes de `message_templates`, clé d'unicité des noms et contrôle des champs.

use mira_vocabulary::{MessageTemplate, MiraError, TemplateId};
use rusqlite::Row;

use super::errors::{name_empty, too_long};

pub(super) const COLUMNS: &str = "id, name, subject, body, created_at, updated_at";

const MAX_NAME: usize = 100;
const MAX_SUBJECT: usize = 300;
const MAX_BODY: usize = 100_000;

/// Lit une ligne dans l'ordre de `COLUMNS`
pub(super) fn template_from_row(row: &Row<'_>) -> rusqlite::Result<MessageTemplate> {
    Ok(MessageTemplate {
        id: TemplateId::from_raw(row.get::<_, String>(0)?),
        name: row.get(1)?,
        subject: row.get(2)?,
        body: row.get(3)?,
        created_at: row.get(4)?,
        updated_at: row.get(5)?,
    })
}

/// Clé d'unicité d'un nom : minuscules, espaces de bord retirés
pub(super) fn name_key(name: &str) -> String {
    name.trim().to_lowercase()
}

/// Refuse un nom vide et tout champ au-delà de sa borne (en caractères)
pub(super) fn check_fields(name: &str, subject: Option<&str>, body: &str) -> Result<(), MiraError> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(name_empty());
    }
    let subject_length = subject.map_or(0, |text| text.chars().count());
    if trimmed.chars().count() > MAX_NAME
        || subject_length > MAX_SUBJECT
        || body.chars().count() > MAX_BODY
    {
        return Err(too_long());
    }
    Ok(())
}
