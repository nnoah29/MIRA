//! COPY et UID COPY (RFC 3501 section 6.4.7, UIDPLUS).

use super::args::text_arg;
use super::transfer::{TransferRequest, transfer};
use crate::imap::capabilities::Capability;
use crate::imap::parser::command::Command;
use crate::imap::response::error::HandlerResult;
use crate::imap::session::state::SessionState;
use crate::imap::session::sync::sync_selected;

/// COPY : duplique les messages dans une autre boîte
pub(crate) fn handle_copy(state: &mut SessionState, command: &Command) -> HandlerResult {
    let (user, from) = state.target()?;
    let request = TransferRequest {
        user,
        from,
        to: text_arg(command, 1)?,
        set: text_arg(command, 0)?,
        by_uid: command.uid,
        remove: false,
    };
    let done = state.ctx.store.write(|data| transfer(data, &request))?;
    let code = if state.ctx.caps.contains(Capability::UidPlus) {
        format!("[{}] ", done.copyuid())
    } else {
        String::new()
    };
    Ok(sync_selected(state).ok(&command.tag, &format!("{code}Copy completed")))
}
