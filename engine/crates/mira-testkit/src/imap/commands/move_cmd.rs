//! MOVE et UID MOVE (RFC 6851).

use super::args::text_arg;
use super::transfer::{TransferRequest, transfer};
use crate::imap::capabilities::Capability;
use crate::imap::parser::command::Command;
use crate::imap::response::error::HandlerResult;
use crate::imap::response::reply::Reply;
use crate::imap::session::state::SessionState;
use crate::imap::session::sync::sync_selected;

/// MOVE : déplace les messages vers une autre boîte
pub(crate) fn handle_move(state: &mut SessionState, command: &Command) -> HandlerResult {
    state.require_cap(Capability::Move)?;
    let (user, from) = state.writable_target()?;
    let request = TransferRequest {
        user,
        from,
        to: text_arg(command, 1)?,
        set: text_arg(command, 0)?,
        by_uid: command.uid,
        remove: true,
    };
    let done = state.ctx.store.write(|data| transfer(data, &request))?;
    let mut reply = Reply::default();
    if state.ctx.caps.contains(Capability::UidPlus) {
        reply.untagged(&format!("OK [{}] Moved", done.copyuid()));
    }
    reply.raw(&sync_selected(state).out);
    Ok(reply.ok(&command.tag, "Move completed"))
}
