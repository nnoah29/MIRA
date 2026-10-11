//! SELECT et EXAMINE avec CONDSTORE et QRESYNC (RFC 7162).

mod announce;
mod params;
mod resync;

use super::args::text_arg;
use crate::imap::capabilities::Capability;
use crate::imap::parser::command::Command;
use crate::imap::response::error::{HandlerResult, ImapError};
use crate::imap::response::reply::Reply;
use crate::imap::session::state::{Selected, SessionState};
use crate::imap::store::names::normalize_name;

/// SELECT : ouvre une boîte en lecture-écriture
pub(crate) fn handle_select(state: &mut SessionState, command: &Command) -> HandlerResult {
    open_mailbox(state, command, false)
}

/// EXAMINE : ouvre une boîte en lecture seule
pub(crate) fn handle_examine(state: &mut SessionState, command: &Command) -> HandlerResult {
    open_mailbox(state, command, true)
}

/// Applique CONDSTORE et vérifie QRESYNC ; renvoie le MODSEQ de départ de la resynchronisation
fn apply_params(state: &mut SessionState, params: &params::SelectParams) -> Result<u64, ImapError> {
    if params.condstore {
        state.require_cap(Capability::Condstore)?;
        state.enabled.condstore = true;
    }
    if params.qresync.is_some() && !state.enabled.qresync {
        return Err(ImapError::bad("QRESYNC is not enabled"));
    }
    Ok(params
        .qresync
        .as_ref()
        .map_or(u64::MAX, |resync| resync.modseq))
}

/// Ouvre la boîte, annonce son état et mémorise la sélection
fn open_mailbox(state: &mut SessionState, command: &Command, read_only: bool) -> HandlerResult {
    let (user, name) = (state.require_user()?, text_arg(command, 0)?);
    state.selected = None;
    let params = params::parse_select_params(command.args.get(1))?;
    let since = apply_params(state, &params)?;
    let view = state
        .ctx
        .store
        .read(|data| data.view(&user, &name, since))
        .ok_or_else(|| ImapError::no("[NONEXISTENT] No such mailbox"))?;
    let mut reply = Reply::default();
    announce::announce_open(
        &mut reply,
        &view,
        state.ctx.caps.contains(Capability::Condstore),
    );
    if let Some(resync) = &params.qresync {
        resync::announce_resync(&mut reply, &view, resync);
    }
    state.selected = Some(Selected {
        name: normalize_name(&name),
        read_only,
        uids: view.uids.clone(),
        synced_modseq: view.highest_modseq,
        validity: view.validity,
    });
    Ok(reply.ok(&command.tag, completion_text(read_only)))
}

/// Texte de fin de commande selon le mode d'ouverture
fn completion_text(read_only: bool) -> &'static str {
    if read_only {
        "[READ-ONLY] Examine completed"
    } else {
        "[READ-WRITE] Select completed"
    }
}
