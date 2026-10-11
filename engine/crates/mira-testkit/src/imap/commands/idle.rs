//! IDLE (RFC 2177).

use crate::imap::capabilities::Capability;
use crate::imap::parser::command::Command;
use crate::imap::response::continuation::Continuation;
use crate::imap::response::error::HandlerResult;
use crate::imap::response::reply::Reply;
use crate::imap::session::state::SessionState;

/// IDLE : annonce l'attente puis laisse la session pousser les changements
pub(crate) fn handle_idle(state: &mut SessionState, command: &Command) -> HandlerResult {
    state.require_cap(Capability::Idle)?;
    let mut reply = Reply::default();
    reply.raw(b"+ idling\r\n");
    let tag = command.tag.clone();
    Ok(reply.with_continuation(Continuation::Idle { tag }))
}
