//! AUTHENTICATE PLAIN et XOAUTH2.

mod sasl;

use super::args::text_arg;
use super::capability::capability_text;
use crate::imap::parser::command::Command;
use crate::imap::parser::token::Token;
use crate::imap::response::continuation::Continuation;
use crate::imap::response::error::{HandlerResult, ImapError};
use crate::imap::response::reply::Reply;
use crate::imap::session::state::SessionState;

/// AUTHENTICATE : accepte la réponse initiale ou demande une suite
pub(crate) fn handle_authenticate(state: &mut SessionState, command: &Command) -> HandlerResult {
    if state.user.is_some() {
        return Err(ImapError::bad("Already authenticated"));
    }
    let mechanism = text_arg(command, 0)?.to_ascii_uppercase();
    if !matches!(mechanism.as_str(), "PLAIN" | "XOAUTH2") {
        return Err(ImapError::no("Unsupported authentication mechanism"));
    }
    if let Some(initial) = command.args.get(1).and_then(Token::text) {
        return Ok(finish_authentication(
            state,
            &command.tag,
            &mechanism,
            &initial,
        ));
    }
    let mut reply = Reply::default();
    reply.raw(b"+ \r\n");
    let tag = command.tag.clone();
    Ok(reply.with_continuation(Continuation::Authenticate { tag, mechanism }))
}

/// Termine l'authentification avec la réponse base64 du client
pub(crate) fn finish_authentication(
    state: &mut SessionState,
    tag: &str,
    mechanism: &str,
    answer: &str,
) -> Reply {
    if answer == "*" {
        return ImapError::bad("Authentication cancelled").into_reply(tag);
    }
    match sasl::verify(mechanism, answer, &state.ctx.credentials) {
        Some(user) => {
            state.ctx.store.write(|data| {
                data.user_mut(&user);
            });
            state.user = Some(user);
            let text = format!("[CAPABILITY {}] Authenticated", capability_text(state));
            Reply::default().ok(tag, &text)
        }
        None => ImapError::no("[AUTHENTICATIONFAILED] Authentication failed").into_reply(tag),
    }
}
