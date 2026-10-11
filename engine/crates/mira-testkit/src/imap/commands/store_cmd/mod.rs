//! STORE et UID STORE, avec UNCHANGEDSINCE et libellés Gmail.

mod apply;
mod spec;

use super::args::text_arg;
use crate::imap::capabilities::Capability;
use crate::imap::parser::command::Command;
use crate::imap::parser::uidset_format::format_uid_set;
use crate::imap::response::error::{HandlerResult, ImapError};
use crate::imap::response::fetch_flags::flags_update;
use crate::imap::response::quoting::astring;
use crate::imap::response::reply::Reply;
use crate::imap::session::state::SessionState;

/// Vérifie les extensions exigées par l'opération
fn check_spec(state: &mut SessionState, spec: &spec::StoreSpec) -> Result<(), ImapError> {
    if spec.labels {
        state.require_cap(Capability::Gmail)?;
    }
    if spec.unchanged_since.is_some() {
        state.require_cap(Capability::Condstore)?;
        state.enabled.condstore = true;
    }
    Ok(())
}

/// Texte de fin de commande, avec la liste des messages refusés par UNCHANGEDSINCE
fn completion_text(modified: &[u32]) -> String {
    if modified.is_empty() {
        return "Store completed".to_string();
    }
    format!(
        "[MODIFIED {}] Conditional STORE failed",
        format_uid_set(modified)
    )
}

/// STORE : modifie drapeaux ou libellés et annonce le résultat
pub(crate) fn handle_store(state: &mut SessionState, command: &Command) -> HandlerResult {
    let (user, mailbox) = state.writable_target()?;
    let spec = spec::parse_store(command)?;
    check_spec(state, &spec)?;
    let set = text_arg(command, 0)?;
    let target = apply::StoreTarget {
        user,
        mailbox,
        set,
        by_uid: command.uid,
    };
    let outcome = state
        .ctx
        .store
        .write(|data| apply::apply_store(data, &target, &spec))?;
    if let Some(selected) = state.selected.as_mut() {
        selected.synced_modseq = outcome.highest;
    }
    let mut reply = Reply::default();
    for row in outcome.rows.iter().filter(|_| !spec.silent) {
        reply.untagged(&row_update(row, spec.labels, state.enabled.condstore));
    }
    Ok(reply.ok(&command.tag, &completion_text(&outcome.modified)))
}

/// Ligne FETCH décrivant le résultat sur un message
fn row_update(row: &crate::imap::store::rows::MessageRow, labels: bool, modseq: bool) -> String {
    if !labels {
        return flags_update(row, modseq);
    }
    let labels: Vec<String> = row.labels.iter().map(|label| astring(label)).collect();
    format!(
        "{} FETCH (UID {} X-GM-LABELS ({}))",
        row.seq,
        row.uid,
        labels.join(" ")
    )
}
