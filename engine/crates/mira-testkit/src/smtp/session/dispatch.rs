//! Aiguillage d'une ligne de commande SMTP vers son gestionnaire.

use std::io;

use super::{Flow, Session};
use crate::smtp::commands::{auth, data, envelope, hello, simple};

/// Route la commande reçue vers le gestionnaire correspondant
pub(crate) async fn dispatch(session: &mut Session, line: &str) -> io::Result<Flow> {
    let (verb, args) = split_command(line);
    match verb.as_str() {
        "EHLO" => hello::ehlo(session).await,
        "HELO" => hello::helo(session).await,
        "AUTH" => auth::handle(session, args).await,
        "MAIL" => envelope::mail_from(session, args).await,
        "RCPT" => envelope::rcpt_to(session, args).await,
        "DATA" => data::handle(session).await,
        "RSET" => envelope::reset(session).await,
        "NOOP" => simple::noop(session).await,
        "QUIT" => simple::quit(session).await,
        _ => simple::unknown(session).await,
    }
}

/// Sépare le verbe (en majuscules) des arguments
fn split_command(line: &str) -> (String, &str) {
    match line.split_once(' ') {
        Some((verb, args)) => (verb.to_ascii_uppercase(), args.trim()),
        None => (line.to_ascii_uppercase(), ""),
    }
}
