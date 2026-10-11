//! LOGIN (RFC 3501 section 6.2.3).

use super::args::text_arg;
use super::capability::capability_text;
use crate::imap::parser::command::Command;
use crate::imap::response::error::{HandlerResult, ImapError};
use crate::imap::response::reply::Reply;
use crate::imap::session::state::SessionState;

/// LOGIN : vérifie le mot de passe dans le registre partagé
pub(crate) fn handle_login(state: &mut SessionState, command: &Command) -> HandlerResult {
    if state.user.is_some() {
        return Err(ImapError::bad("Already authenticated"));
    }
    let (user, password) = (text_arg(command, 0)?, text_arg(command, 1)?);
    if !state.ctx.credentials.check_password(&user, &password) {
        return Err(ImapError::no("[AUTHENTICATIONFAILED] Invalid credentials"));
    }
    state.ctx.store.write(|data| {
        data.user_mut(&user);
    });
    state.user = Some(user);
    let text = format!("[CAPABILITY {}] Logged in", capability_text(state));
    Ok(Reply::default().ok(&command.tag, &text))
}
