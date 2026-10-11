//! APPEND (RFC 3501 section 6.3.11, LITERAL+ et UIDPLUS).

use super::append_parts::{AppendParts, parse_append_parts};
use super::args::text_arg;
use crate::imap::capabilities::Capability;
use crate::imap::parser::command::Command;
use crate::imap::response::error::{HandlerResult, ImapError};
use crate::imap::session::state::SessionState;
use crate::imap::session::sync::sync_selected;
use crate::imap::store::data::StoreData;

/// Dépose le message dans la boîte ; renvoie UIDVALIDITY et l'UID attribué
fn store_message(
    data: &mut StoreData,
    user: &str,
    mailbox: &str,
    parts: &AppendParts,
) -> Option<(u32, u32)> {
    data.mailbox(user, mailbox)?;
    let body = data.new_body(&parts.data);
    let uid = data.deliver_body(user, mailbox, &body, &parts.flags, parts.date);
    let validity = data.mailbox(user, mailbox)?.uidvalidity;
    Some((validity, uid))
}

/// APPEND : ajoute un message fourni par le client
pub(crate) fn handle_append(state: &mut SessionState, command: &Command) -> HandlerResult {
    let user = state.require_user()?;
    let mailbox = text_arg(command, 0)?;
    let parts = parse_append_parts(command.args.get(1..).unwrap_or(&[]))?;
    let stored = state
        .ctx
        .store
        .write(|data| store_message(data, &user, &mailbox, &parts));
    let (validity, uid) = stored.ok_or_else(|| ImapError::no("[TRYCREATE] No such mailbox"))?;
    let code = if state.ctx.caps.contains(Capability::UidPlus) {
        format!("[APPENDUID {validity} {uid}] ")
    } else {
        String::new()
    };
    Ok(sync_selected(state).ok(&command.tag, &format!("{code}Append completed")))
}
