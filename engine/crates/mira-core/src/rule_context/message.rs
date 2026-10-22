//! Contexte d'évaluation d'un message stocké : adresses du compte, pièces jointes, fil existant.

use mira_vocabulary::{AccountId, EmailAddress, MessageSummary, MiraError, Timestamp};

use crate::rules::context::{AttachmentInfo, MessageContext};
use crate::storage::Tx;

/// Adresses désignant « moi » pour ce compte : l'adresse du compte puis celles de ses identités, sans doublon
pub fn own_addresses(tx: &Tx<'_>, account: &AccountId) -> Result<Vec<EmailAddress>, MiraError> {
    let mut addresses = Vec::new();
    if let Some(found) = tx.get_account(account)? {
        addresses.push(found.address);
    }
    for identity in tx.list_identities(account)? {
        if !addresses.contains(&identity.address) {
            addresses.push(identity.address);
        }
    }
    Ok(addresses)
}

/// Contexte d'un message stocké ; les pièces intégrées au corps (images en ligne) ne comptent pas
pub fn message_context(
    tx: &Tx<'_>,
    summary: MessageSummary,
    own_addresses: Vec<EmailAddress>,
    now: Timestamp,
) -> Result<MessageContext, MiraError> {
    let attachments = tx
        .list_attachments(&summary.id)?
        .into_iter()
        .filter(|attachment| !attachment.inline)
        .map(|attachment| AttachmentInfo {
            name: attachment.filename.unwrap_or_default(),
            media_type: attachment.mime_type,
        })
        .collect();
    let in_existing_thread = match &summary.thread {
        Some(thread) => tx.thread_has_other_messages(thread, &summary.id)?,
        None => false,
    };
    Ok(MessageContext {
        summary,
        own_addresses,
        attachments,
        in_existing_thread,
        now,
    })
}
