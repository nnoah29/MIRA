//! STATUS (RFC 3501 section 6.3.10).

use super::args::{list_arg, text_arg};
use crate::imap::capabilities::Capability;
use crate::imap::parser::command::Command;
use crate::imap::parser::token::Token;
use crate::imap::response::error::{HandlerResult, ImapError};
use crate::imap::response::quoting::quote;
use crate::imap::response::reply::Reply;
use crate::imap::session::state::SessionState;
use crate::imap::store::view::MailboxView;

/// STATUS : compteurs d'une boîte sans la sélectionner
pub(crate) fn handle_status(state: &mut SessionState, command: &Command) -> HandlerResult {
    let user = state.require_user()?;
    let name = text_arg(command, 0)?;
    let items = list_arg(command, 1)?;
    let view = state
        .ctx
        .store
        .read(|data| data.view(&user, &name, u64::MAX))
        .ok_or_else(|| ImapError::no("[NONEXISTENT] No such mailbox"))?;
    let condstore = state.ctx.caps.contains(Capability::Condstore);
    let parts: Vec<String> = items
        .iter()
        .filter_map(Token::keyword)
        .filter_map(|item| status_item(&item, &view, condstore))
        .collect();
    let mut reply = Reply::default();
    reply.untagged(&format!("STATUS {} ({})", quote(&name), parts.join(" ")));
    Ok(reply.ok(&command.tag, "STATUS completed"))
}

/// Valeur d'un élément STATUS demandé
fn status_item(item: &str, view: &MailboxView, condstore: bool) -> Option<String> {
    match item {
        "MESSAGES" => Some(format!("MESSAGES {}", view.uids.len())),
        "RECENT" => Some("RECENT 0".to_string()),
        "UIDNEXT" => Some(format!("UIDNEXT {}", view.uidnext)),
        "UIDVALIDITY" => Some(format!("UIDVALIDITY {}", view.validity)),
        "UNSEEN" => Some(format!("UNSEEN {}", view.unseen)),
        "HIGHESTMODSEQ" if condstore => Some(format!("HIGHESTMODSEQ {}", view.highest_modseq)),
        _ => None,
    }
}
