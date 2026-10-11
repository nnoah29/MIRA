//! Commandes sans argument : NOOP, QUIT et commande inconnue.

use std::io;

use crate::smtp::session::{Flow, Session};

/// Répond à NOOP
pub(crate) async fn noop(session: &mut Session) -> io::Result<Flow> {
    session.answer("250 2.0.0 OK").await
}

/// Répond à QUIT puis ferme la connexion
pub(crate) async fn quit(session: &mut Session) -> io::Result<Flow> {
    session.reply("221 2.0.0 Bye").await?;
    Ok(Flow::Close)
}

/// Refuse une commande inconnue
pub(crate) async fn unknown(session: &mut Session) -> io::Result<Flow> {
    session.answer("500 5.5.2 Command not recognized").await
}
