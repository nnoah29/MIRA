//! Retrouve où un message vit sur le serveur : compte, boîte et UID d'une apparition valide.

use mira_vocabulary::error::codes::not_found;
use mira_vocabulary::{Account, ErrorCategory, MessageId, MessageSummary, MiraError};

use crate::accounts::require_account;
use crate::storage::Tx;
use crate::storage::repositories::messages::message_not_found;

/// Position d'un message sur le serveur.
#[derive(Debug, Clone)]
pub struct SourceLocation {
    pub message: MessageSummary,
    pub account: Account,
    pub mailbox_name: String,
    pub uid: u32,
    pub validity: Option<u32>,
}

/// Cherche une apparition non supprimée, à l'UID fiable (ni nul ni à revalider) ; sinon `not_found.message`
pub fn locate_message(tx: &Tx<'_>, id: &MessageId) -> Result<SourceLocation, MiraError> {
    let message = tx.get_message(id)?.ok_or_else(|| message_not_found(id))?;
    let account = require_account(tx, &message.account)?;
    for appearance in tx.appearances_of_message(id)? {
        if appearance.deleted || appearance.uid == 0 || appearance.needs_revalidation {
            continue;
        }
        if let Some(mailbox) = tx.get_mailbox(&appearance.mailbox)? {
            let validity = mailbox.marker.map(|marker| marker.uid_validity);
            return Ok(SourceLocation {
                message,
                account,
                mailbox_name: mailbox.name,
                uid: appearance.uid,
                validity,
            });
        }
    }
    Err(MiraError::new(
        ErrorCategory::NotFound,
        not_found::MESSAGE,
        "aucune copie du message n'existe sur le serveur",
    ))
}
