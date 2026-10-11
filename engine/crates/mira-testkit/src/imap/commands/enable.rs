//! ENABLE (RFC 5161) pour CONDSTORE et QRESYNC.

use crate::imap::capabilities::Capability;
use crate::imap::parser::command::Command;
use crate::imap::response::error::HandlerResult;
use crate::imap::response::reply::Reply;
use crate::imap::session::state::SessionState;

/// ENABLE : active CONDSTORE ou QRESYNC si le serveur les annonce
pub(crate) fn handle_enable(state: &mut SessionState, command: &Command) -> HandlerResult {
    state.require_cap(Capability::Enable)?;
    let caps = state.ctx.caps.clone();
    let mut granted = Vec::new();
    for token in &command.args {
        match token.keyword().as_deref() {
            Some("CONDSTORE") if caps.contains(Capability::Condstore) => {
                state.enabled.condstore = true;
                granted.push("CONDSTORE");
            }
            Some("QRESYNC") if caps.contains(Capability::Qresync) => {
                state.enabled.condstore = true;
                state.enabled.qresync = true;
                granted.push("QRESYNC");
            }
            _ => {}
        }
    }
    let mut reply = Reply::default();
    if !granted.is_empty() {
        reply.untagged(&format!("ENABLED {}", granted.join(" ")));
    }
    Ok(reply.ok(&command.tag, "ENABLE completed"))
}
