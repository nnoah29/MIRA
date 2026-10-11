//! EXPUNGE, UID EXPUNGE et CLOSE.

use super::args::text_arg;
use crate::imap::capabilities::Capability;
use crate::imap::parser::command::Command;
use crate::imap::parser::uidset::resolve_set;
use crate::imap::response::error::{HandlerResult, ImapError};
use crate::imap::response::reply::Reply;
use crate::imap::session::state::SessionState;
use crate::imap::session::sync::sync_selected;
use crate::imap::store::data::StoreData;

/// Supprime les messages marqués `\Deleted` (dans l'ensemble donné, s'il y en a un)
fn remove_deleted(
    data: &mut StoreData,
    user: &str,
    mailbox: &str,
    set: Option<&str>,
) -> Result<(), ImapError> {
    let Some(found) = data.mailbox_mut(user, mailbox) else {
        return Ok(());
    };
    let ids = found.uids();
    let allowed = match set {
        Some(text) => resolve_set(text, &ids, true)?,
        None => ids,
    };
    let is_deleted = |flags: &[String]| flags.iter().any(|f| f.eq_ignore_ascii_case("\\Deleted"));
    let doomed: Vec<u32> = found
        .entries
        .iter()
        .filter(|entry| allowed.contains(&entry.uid) && is_deleted(&entry.flags))
        .map(|entry| entry.uid)
        .collect();
    found.remove_uids(&doomed);
    Ok(())
}

/// EXPUNGE : supprime définitivement les messages marqués `\Deleted`
pub(crate) fn handle_expunge(state: &mut SessionState, command: &Command) -> HandlerResult {
    let (user, mailbox) = state.writable_target()?;
    let set = if command.uid {
        state.require_cap(Capability::UidPlus)?;
        Some(text_arg(command, 0)?)
    } else {
        None
    };
    let store = state.ctx.store.clone();
    store.write(|data| remove_deleted(data, &user, &mailbox, set.as_deref()))?;
    Ok(sync_selected(state).ok(&command.tag, "Expunge completed"))
}

/// CLOSE : supprime silencieusement les messages `\Deleted` puis désélectionne
pub(crate) fn handle_close(state: &mut SessionState, command: &Command) -> HandlerResult {
    let read_only = state.require_selected()?.read_only;
    if !read_only {
        let (user, mailbox) = state.target()?;
        let store = state.ctx.store.clone();
        store.write(|data| remove_deleted(data, &user, &mailbox, None))?;
    }
    state.selected = None;
    Ok(Reply::default().ok(&command.tag, "Close completed"))
}
