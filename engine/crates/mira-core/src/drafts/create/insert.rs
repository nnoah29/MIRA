//! Enregistrement d'un brouillon neuf et adresses propres de l'utilisateur.

use mira_vocabulary::{AccountId, Draft, DraftId, DraftState, Identity, MiraError};

use super::fields::DraftFields;
use crate::actions::QueueContext;
use crate::storage::Tx;

/// Insère un brouillon en édition, sans pièce jointe, avec un identifiant frais
pub(super) fn insert_new_draft(
    tx: &Tx<'_>,
    ctx: &QueueContext<'_>,
    account: &AccountId,
    identity: &Identity,
    fields: DraftFields,
) -> Result<Draft, MiraError> {
    let draft = Draft {
        id: DraftId::from_raw(ctx.ids.next_id()),
        account: account.clone(),
        identity: identity.id.clone(),
        to: fields.to,
        cc: fields.cc,
        bcc: Vec::new(),
        subject: fields.subject,
        body_text: String::new(),
        body_html: None,
        attachments: Vec::new(),
        in_reply_to: fields.in_reply_to,
        state: DraftState::Editing,
        updated_at: ctx.clock.now(),
    };
    tx.insert_draft(&draft)?;
    Ok(draft)
}

/// Adresses de l'utilisateur sur ce compte : son adresse et celles de ses identités
pub(super) fn own_addresses(tx: &Tx<'_>, account: &AccountId) -> Result<Vec<String>, MiraError> {
    let mut own: Vec<String> = tx
        .list_identities(account)?
        .into_iter()
        .map(|identity| identity.address.as_str().to_string())
        .collect();
    if let Some(found) = tx.get_account(account)? {
        own.push(found.address.as_str().to_string());
    }
    Ok(own)
}
