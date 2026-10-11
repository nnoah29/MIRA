//! CREATE, DELETE et RENAME.

use super::args::text_arg;
use crate::imap::parser::command::Command;
use crate::imap::response::error::{HandlerResult, ImapError};
use crate::imap::response::reply::Reply;
use crate::imap::session::state::SessionState;

/// CREATE : crée une boîte
pub(crate) fn handle_create(state: &mut SessionState, command: &Command) -> HandlerResult {
    let (user, name) = (state.require_user()?, text_arg(command, 0)?);
    let created = state
        .ctx
        .store
        .write(|data| data.create_named(&user, &name));
    if !created {
        return Err(ImapError::no("[ALREADYEXISTS] Mailbox already exists"));
    }
    Ok(Reply::default().ok(&command.tag, "CREATE completed"))
}

/// DELETE : supprime une boîte et ses messages
pub(crate) fn handle_delete(state: &mut SessionState, command: &Command) -> HandlerResult {
    let (user, name) = (state.require_user()?, text_arg(command, 0)?);
    let deleted = state
        .ctx
        .store
        .write(|data| data.delete_named(&user, &name));
    if !deleted {
        return Err(ImapError::no("[NONEXISTENT] Cannot delete this mailbox"));
    }
    Ok(Reply::default().ok(&command.tag, "DELETE completed"))
}

/// RENAME : renomme une boîte et ses sous-boîtes
pub(crate) fn handle_rename(state: &mut SessionState, command: &Command) -> HandlerResult {
    let (user, from) = (state.require_user()?, text_arg(command, 0)?);
    let to = text_arg(command, 1)?;
    let outcome = state
        .ctx
        .store
        .write(|data| data.rename_named(&user, &from, &to));
    outcome.map_err(ImapError::no)?;
    Ok(Reply::default().ok(&command.tag, "RENAME completed"))
}
