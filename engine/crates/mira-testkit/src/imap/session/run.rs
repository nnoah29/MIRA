//! Boucle principale d'une session.

use std::io;
use std::sync::Arc;

use tokio::net::TcpStream;

use super::flow::continue_flow;
use super::log::{failure_keys, log_line};
use super::reader::{RawCommand, read_command};
use super::state::SessionState;
use super::transport::Transport;
use crate::imap::commands::capability::capability_text;
use crate::imap::commands::router::dispatch;
use crate::imap::parser::command::{Command, parse_command};
use crate::imap::response::error::ImapError;
use crate::imap::response::reply::Reply;
use crate::imap::server::context::ServerContext;

/// Sert une connexion jusqu'à sa fermeture
pub(crate) async fn run_session(stream: TcpStream, context: Arc<ServerContext>) {
    let mut io = Transport::new(stream);
    let mut state = SessionState::new(context);
    let _ = serve(&mut io, &mut state).await;
}

/// Envoie la bannière puis traite les commandes une à une
async fn serve(io: &mut Transport, state: &mut SessionState) -> io::Result<()> {
    let greeting = format!(
        "* OK [CAPABILITY {}] MIRA fake IMAP ready\r\n",
        capability_text(state)
    );
    io.send(greeting.as_bytes()).await?;
    while let Some(raw) = read_command(io, &state.ctx.caps).await? {
        let reply = process(state, &raw);
        io.send(&reply.out).await?;
        if let Some(next) = reply.continuation {
            continue_flow(io, state, next).await?;
        }
        if reply.close {
            break;
        }
    }
    Ok(())
}

/// Analyse une commande brute puis l'exécute
fn process(state: &mut SessionState, raw: &RawCommand) -> Reply {
    let tag = raw.first_line.split_whitespace().next().unwrap_or("*");
    if let Some(message) = &raw.error {
        return ImapError::bad(message).into_reply(tag);
    }
    match parse_command(&raw.bytes, &raw.first_line) {
        Ok(command) => run_command(state, &command),
        Err(error) => ImapError::from(error).into_reply(tag),
    }
}

/// Journalise la commande, applique une panne programmée ou l'exécute
fn run_command(state: &mut SessionState, command: &Command) -> Reply {
    let store = state.ctx.store.clone();
    store.record_command(log_line(command));
    match store.take_failure(&failure_keys(command)) {
        Some(response) => {
            let mut reply = Reply::default();
            reply.raw(format!("{} {response}\r\n", command.tag).as_bytes());
            reply
        }
        None => dispatch(state, command),
    }
}
