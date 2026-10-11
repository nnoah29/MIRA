//! SEARCH et UID SEARCH, avec MODSEQ, HEADER et ESEARCH.

mod matching;
mod matching_text;
mod results;
mod run;

use crate::imap::capabilities::Capability;
use crate::imap::parser::command::Command;
use crate::imap::parser::search::parse_search;
use crate::imap::response::error::HandlerResult;
use crate::imap::response::reply::Reply;
use crate::imap::session::state::SessionState;

/// SEARCH : liste les messages qui satisfont tous les critères
pub(crate) fn handle_search(state: &mut SessionState, command: &Command) -> HandlerResult {
    let (user, mailbox) = state.target()?;
    let query = parse_search(&command.args)?;
    if query.returns.is_some() {
        state.require_cap(Capability::Esearch)?;
    }
    let uses_modseq = run::contains_modseq(&query.criteria);
    if uses_modseq {
        state.require_cap(Capability::Condstore)?;
        state.enabled.condstore = true;
    }
    let hits = state
        .ctx
        .store
        .read(|data| run::run_search(data, &user, &mailbox, &query.criteria, command.uid));
    let mut reply = Reply::default();
    reply.untagged(&results::format_results(
        &hits,
        &query,
        command,
        uses_modseq,
    ));
    Ok(reply.ok(&command.tag, "Search completed"))
}
