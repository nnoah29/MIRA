//! Création d'un brouillon : neuf, réponse, réponse à tous ou transfert.

use mira_vocabulary::{AccountId, IdentityId, MessageId, MiraError};

use crate::actions::errors::input::invalid_parameter;
use crate::api::contract::{DraftMode, Response, WriteResponse};
use crate::api::engine::Engine;
use crate::drafts::{create_blank, create_forward, create_reply, create_reply_all};
use crate::storage::repositories::messages::message_not_found;

/// Crée le brouillon ; un message d'origine doit appartenir au compte indiqué
pub(crate) fn handle_create_draft(
    engine: &Engine,
    account: &AccountId,
    identity: Option<&IdentityId>,
    mode: DraftMode,
) -> Result<Response, MiraError> {
    let ctx = engine.queue_context();
    let draft = match mode {
        DraftMode::New => create_blank(&ctx, account, identity)?,
        DraftMode::Reply { message } => {
            require_message_of(engine, &message, account)?;
            create_reply(&ctx, &message, identity)?
        }
        DraftMode::ReplyAll { message } => {
            require_message_of(engine, &message, account)?;
            create_reply_all(&ctx, &message, identity)?
        }
        DraftMode::Forward { message } => {
            require_message_of(engine, &message, account)?;
            create_forward(&ctx, &message, identity)?
        }
    };
    Ok(Response::Write(WriteResponse::Draft(Box::new(draft))))
}

/// Exige que le message d'origine existe et appartienne au compte du brouillon
fn require_message_of(
    engine: &Engine,
    message: &MessageId,
    account: &AccountId,
) -> Result<(), MiraError> {
    let found = engine
        .database
        .read(|tx| tx.get_message(message))?
        .ok_or_else(|| message_not_found(message))?;
    if &found.account != account {
        return Err(invalid_parameter(
            "le message d'origine appartient à un autre compte",
        ));
    }
    Ok(())
}
