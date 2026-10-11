//! CAPABILITY, NOOP et LOGOUT.

use crate::imap::parser::command::Command;
use crate::imap::response::error::HandlerResult;
use crate::imap::response::reply::Reply;
use crate::imap::session::state::SessionState;
use crate::imap::session::sync::sync_selected;

/// Texte de la liste de capacités selon l'état d'authentification
pub(crate) fn capability_text(state: &SessionState) -> String {
    let mut words = vec!["IMAP4rev1"];
    if state.user.is_none() {
        words.extend(["AUTH=PLAIN", "AUTH=XOAUTH2"]);
    }
    words.extend(state.ctx.caps.tokens());
    words.join(" ")
}

/// CAPABILITY : liste les extensions annoncées
pub(crate) fn handle_capability(state: &mut SessionState, command: &Command) -> HandlerResult {
    let mut reply = Reply::default();
    reply.untagged(&format!("CAPABILITY {}", capability_text(state)));
    Ok(reply.ok(&command.tag, "CAPABILITY completed"))
}

/// NOOP : renvoie les changements survenus dans la boîte sélectionnée
pub(crate) fn handle_noop(state: &mut SessionState, command: &Command) -> HandlerResult {
    Ok(sync_selected(state).ok(&command.tag, "NOOP completed"))
}

/// LOGOUT : annonce BYE puis ferme la connexion
pub(crate) fn handle_logout(_state: &mut SessionState, command: &Command) -> HandlerResult {
    let mut reply = Reply::default();
    reply.untagged("BYE Logging out");
    Ok(reply.ok(&command.tag, "LOGOUT completed").closing())
}
