//! Requête de liste d'une boîte : le plan dépend de sa taille pour rester rapide sur de gros miroirs.

use mira_vocabulary::{MailboxId, MiraError};
use rusqlite::params;

use super::builder::QueryBuilder;
use crate::storage::Tx;
use crate::storage::errors::storage_error;
use crate::storage::repositories::label_settings::hidden::hidden_from_inbox;

const LARGE_MAILBOX: i64 = 2_000;

impl Tx<'_> {
    /// Requête d'une boîte : parcours des apparitions si elle est petite, parcours par date si elle est grosse
    pub(super) fn mailbox_builder(&self, mailbox: &MailboxId) -> Result<QueryBuilder, MiraError> {
        let size: i64 = self
            .sql()
            .query_row(
                "SELECT COUNT(*) FROM (SELECT 1 FROM appearances WHERE mailbox_id = ?1 LIMIT ?2)",
                params![mailbox.as_str(), LARGE_MAILBOX + 1],
                |row| row.get(0),
            )
            .map_err(storage_error)?;
        let mut builder = match size > LARGE_MAILBOX {
            true => builder_by_date(mailbox),
            false => builder_by_appearances(mailbox),
        };
        if self.is_inbox(mailbox)? {
            builder.require(format!("NOT {}", hidden_from_inbox()));
        }
        Ok(builder)
    }
}

impl Tx<'_> {
    /// Vrai si la boîte est une Réception
    fn is_inbox(&self, mailbox: &MailboxId) -> Result<bool, MiraError> {
        self.sql()
            .query_row(
                "SELECT role = 'inbox' FROM mailboxes WHERE id = ?1",
                [mailbox.as_str()],
                |row| row.get(0),
            )
            .or_else(|error| match error {
                rusqlite::Error::QueryReturnedNoRows => Ok(false),
                other => Err(other),
            })
            .map_err(storage_error)
    }
}

/// Part des apparitions de la boîte, puis cherche le message de chacune (petite boîte)
fn builder_by_appearances(mailbox: &MailboxId) -> QueryBuilder {
    let mut builder =
        QueryBuilder::new("appearances a CROSS JOIN messages m ON m.id = a.message_id");
    let bound = builder.bind(mailbox.as_str().to_string());
    builder.require(format!("a.mailbox_id = {bound}"));
    builder.require("a.deleted = 0");
    builder
}

/// Parcourt les messages du plus récent au plus ancien en vérifiant leur apparition (grosse boîte)
fn builder_by_date(mailbox: &MailboxId) -> QueryBuilder {
    let mut builder = QueryBuilder::new("messages m");
    let bound = builder.bind(mailbox.as_str().to_string());
    builder.require(format!(
        "EXISTS (SELECT 1 FROM appearances a WHERE a.message_id = m.id AND a.mailbox_id = {bound} AND a.deleted = 0)"
    ));
    builder
}
