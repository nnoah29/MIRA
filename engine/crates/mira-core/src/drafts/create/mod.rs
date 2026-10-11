//! Création d'un brouillon : neuf, réponse, réponse à tous, transfert.

mod fields;
mod insert;

use mira_vocabulary::{AccountId, Draft, IdentityId, MessageId, MiraError};

use super::identity::resolve_identity;
use crate::actions::QueueContext;
use crate::actions::errors::not_found::account_not_found;
use crate::storage::repositories::messages::message_not_found;
use fields::{DraftFields, Mode, fields_from};
use insert::{insert_new_draft, own_addresses};

/// Crée un brouillon vide ; sans identité demandée, celle par défaut du compte est utilisée
pub fn create_blank(
    ctx: &QueueContext<'_>,
    account: &AccountId,
    identity: Option<&IdentityId>,
) -> Result<Draft, MiraError> {
    ctx.db.write(|tx| {
        if tx.get_account(account)?.is_none() {
            return Err(account_not_found(account));
        }
        let identity = resolve_identity(tx, account, identity)?;
        insert_new_draft(tx, ctx, account, &identity, DraftFields::default())
    })
}

/// Crée la réponse à un message : destinataire, objet `Re:` et lien vers l'original
pub fn create_reply(
    ctx: &QueueContext<'_>,
    original: &MessageId,
    identity: Option<&IdentityId>,
) -> Result<Draft, MiraError> {
    create_from_message(ctx, original, identity, Mode::Reply)
}

/// Crée la réponse à tous : les autres destinataires d'origine passent en copie
pub fn create_reply_all(
    ctx: &QueueContext<'_>,
    original: &MessageId,
    identity: Option<&IdentityId>,
) -> Result<Draft, MiraError> {
    create_from_message(ctx, original, identity, Mode::ReplyAll)
}

/// Crée le transfert d'un message : sans destinataire, objet `Fwd:` et lien vers l'original
pub fn create_forward(
    ctx: &QueueContext<'_>,
    original: &MessageId,
    identity: Option<&IdentityId>,
) -> Result<Draft, MiraError> {
    create_from_message(ctx, original, identity, Mode::Forward)
}

/// Crée un brouillon dans le compte du message d'origine, selon le mode
fn create_from_message(
    ctx: &QueueContext<'_>,
    original: &MessageId,
    identity: Option<&IdentityId>,
    mode: Mode,
) -> Result<Draft, MiraError> {
    ctx.db.write(|tx| {
        let message = tx
            .get_message(original)?
            .ok_or_else(|| message_not_found(original))?;
        let identity = resolve_identity(tx, &message.account, identity)?;
        let own = own_addresses(tx, &message.account)?;
        let fields = fields_from(mode, &message, &own);
        insert_new_draft(tx, ctx, &message.account, &identity, fields)
    })
}
