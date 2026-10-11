//! Suites d'échange : authentification en deux temps et IDLE.

use std::io;

use super::idle::run_idle;
use super::state::SessionState;
use super::transport::Transport;
use crate::imap::commands::authenticate::finish_authentication;
use crate::imap::response::continuation::Continuation;

/// Poursuit le dialogue demandé par la réponse précédente
pub(crate) async fn continue_flow(
    io: &mut Transport,
    state: &mut SessionState,
    next: Continuation,
) -> io::Result<()> {
    match next {
        Continuation::Idle { tag } => run_idle(io, state, &tag).await,
        Continuation::Authenticate { tag, mechanism } => {
            let mut line = Vec::new();
            if io.read_line_into(&mut line).await? == 0 {
                return Err(io::ErrorKind::UnexpectedEof.into());
            }
            let answer = String::from_utf8_lossy(&line).trim().to_string();
            let reply = finish_authentication(state, &tag, &mechanism, &answer);
            io.send(&reply.out).await
        }
    }
}
