//! Écriture, lecture et vidage du cache d'analyse des messages.

use mira_vocabulary::{AccountId, MessageId, MiraError};
use rusqlite::{OptionalExtension, params};

use crate::storage::Tx;
use crate::storage::errors::storage_error;

/// Contenu extrait d'un message : texte brut et HTML éventuel (non assaini).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedContent {
    pub text: String,
    pub html: Option<String>,
}

impl Tx<'_> {
    /// Enregistre (ou remplace) le contenu extrait d'un message
    pub fn store_parsed(
        &self,
        message: &MessageId,
        text: &str,
        html: Option<&str>,
    ) -> Result<(), MiraError> {
        self.sql()
            .execute(
                "INSERT OR REPLACE INTO parsed_cache (message_id, text_body, html_body) VALUES (?1, ?2, ?3)",
                params![message.as_str(), text, html],
            )
            .map_err(storage_error)?;
        Ok(())
    }

    /// Relit le contenu extrait d'un message, ou rien s'il n'est pas en cache
    pub fn load_parsed(&self, message: &MessageId) -> Result<Option<ParsedContent>, MiraError> {
        self.sql()
            .query_row(
                "SELECT text_body, html_body FROM parsed_cache WHERE message_id = ?1",
                [message.as_str()],
                |row| {
                    Ok(ParsedContent {
                        text: row.get(0)?,
                        html: row.get(1)?,
                    })
                },
            )
            .optional()
            .map_err(storage_error)
    }

    /// Vide le cache d'analyse des messages d'un compte
    pub fn clear_parsed_cache(&self, account: &AccountId) -> Result<(), MiraError> {
        self.sql()
            .execute(
                "DELETE FROM parsed_cache WHERE message_id IN (SELECT id FROM messages WHERE account_id = ?1)",
                [account.as_str()],
            )
            .map_err(storage_error)?;
        Ok(())
    }
}
