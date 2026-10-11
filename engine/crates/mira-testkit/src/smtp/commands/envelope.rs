//! Enveloppe SMTP : MAIL FROM, RCPT TO et RSET (RFC 5321 4.1.1.2 à 4.1.1.5).

use std::io;

use crate::smtp::session::{Flow, Session};

const AUTH_REQUIRED: &str = "530 5.7.0 Authentication required";
const BAD_SEQUENCE: &str = "503 5.5.1 Bad sequence of commands";

/// Ouvre une transaction avec l'expéditeur donné
pub(crate) async fn mail_from(session: &mut Session, args: &str) -> io::Result<Flow> {
    if session.transaction.user.is_none() {
        return session.answer(AUTH_REQUIRED).await;
    }
    if session.transaction.from.is_some() {
        return session.answer(BAD_SEQUENCE).await;
    }
    let Some(address) = parse_path(args, "FROM:") else {
        return session.answer("501 5.1.7 Syntax error in MAIL FROM").await;
    };
    if let Some(failure) = session.take_failure() {
        return session.answer(&failure).await;
    }
    session.transaction.from = Some(address);
    session.answer("250 2.1.0 OK").await
}

/// Ajoute un destinataire à la transaction en cours
pub(crate) async fn rcpt_to(session: &mut Session, args: &str) -> io::Result<Flow> {
    if session.transaction.from.is_none() {
        return session.answer(BAD_SEQUENCE).await;
    }
    let Some(address) = parse_path(args, "TO:").filter(|path| !path.is_empty()) else {
        return session.answer("501 5.1.3 Syntax error in RCPT TO").await;
    };
    if let Some(failure) = session.take_failure() {
        return session.answer(&failure).await;
    }
    session.transaction.recipients.push(address);
    session.answer("250 2.1.5 OK").await
}

/// Abandonne la transaction en cours
pub(crate) async fn reset(session: &mut Session) -> io::Result<Flow> {
    session.transaction.reset();
    session.answer("250 2.0.0 OK").await
}

/// Extrait l'adresse entre chevrons après un préfixe comme `FROM:`
fn parse_path(args: &str, prefix: &str) -> Option<String> {
    let head = args.get(..prefix.len())?;
    if !head.eq_ignore_ascii_case(prefix) {
        return None;
    }
    let rest = args.get(prefix.len()..)?.trim_start();
    let (address, _) = rest.strip_prefix('<')?.split_once('>')?;
    Some(address.to_string())
}
