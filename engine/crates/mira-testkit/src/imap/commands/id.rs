//! ID (RFC 2971).

use crate::imap::capabilities::Capability;
use crate::imap::parser::command::Command;
use crate::imap::response::error::HandlerResult;
use crate::imap::response::reply::Reply;
use crate::imap::session::state::SessionState;

/// ID : renvoie l'identité du faux serveur
pub(crate) fn handle_id(state: &mut SessionState, command: &Command) -> HandlerResult {
    state.require_cap(Capability::Id)?;
    let mut reply = Reply::default();
    reply.untagged("ID (\"name\" \"mira-fake-imap\" \"version\" \"0.1\")");
    Ok(reply.ok(&command.tag, "ID completed"))
}
