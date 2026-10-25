//! Écritures sur les modèles de messages : création, mise à jour, suppression.

use mira_vocabulary::{MessageTemplate, MiraError, TemplateId};
use rusqlite::{OptionalExtension, params};

use super::errors::{name_taken, template_not_found};
use super::rows::{check_fields, name_key};
use crate::storage::Tx;
use crate::storage::errors::storage_error;

impl Tx<'_> {
    /// Insère un modèle ; nom vide, champ trop long ou nom déjà pris (casse ignorée) sont refusés
    pub fn insert_template(&self, template: &MessageTemplate) -> Result<(), MiraError> {
        check_fields(&template.name, template.subject.as_deref(), &template.body)?;
        self.ensure_template_name_free(&template.name, None)?;
        self.sql()
            .execute(
                "INSERT INTO message_templates (id, name, name_key, subject, body, created_at, updated_at) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    template.id.as_str(),
                    template.name.trim(),
                    name_key(&template.name),
                    template.subject,
                    template.body,
                    template.created_at,
                    template.updated_at
                ],
            )
            .map_err(storage_error)?;
        Ok(())
    }

    /// Remplace nom, objet et corps d'un modèle ; inconnu : `not_found.template`
    pub fn update_template(
        &self,
        id: &TemplateId,
        name: &str,
        subject: Option<&str>,
        body: &str,
        now: i64,
    ) -> Result<(), MiraError> {
        check_fields(name, subject, body)?;
        self.ensure_template_name_free(name, Some(id))?;
        let changed = self
            .sql()
            .execute(
                "UPDATE message_templates SET name = ?2, name_key = ?3, subject = ?4, body = ?5, updated_at = ?6 \
                 WHERE id = ?1",
                params![id.as_str(), name.trim(), name_key(name), subject, body, now],
            )
            .map_err(storage_error)?;
        if changed == 0 {
            return Err(template_not_found());
        }
        Ok(())
    }

    /// Supprime un modèle et les copies de son texte gardées pour l'idempotence ; inconnu : `not_found.template`
    pub fn delete_template(&self, id: &TemplateId) -> Result<(), MiraError> {
        let changed = self
            .sql()
            .execute("DELETE FROM message_templates WHERE id = ?1", [id.as_str()])
            .map_err(storage_error)?;
        if changed == 0 {
            return Err(template_not_found());
        }
        self.sql()
            .execute(
                "DELETE FROM template_idempotency WHERE json_extract(result, '$.id') = ?1",
                [id.as_str()],
            )
            .map_err(storage_error)?;
        Ok(())
    }

    /// Refuse un nom déjà porté par un autre modèle que `except`
    fn ensure_template_name_free(
        &self,
        name: &str,
        except: Option<&TemplateId>,
    ) -> Result<(), MiraError> {
        let owner: Option<String> = self
            .sql()
            .query_row(
                "SELECT id FROM message_templates WHERE name_key = ?1",
                [name_key(name)],
                |row| row.get(0),
            )
            .optional()
            .map_err(storage_error)?;
        match owner {
            Some(found) if except.is_none_or(|id| id.as_str() != found) => Err(name_taken()),
            _ => Ok(()),
        }
    }
}
