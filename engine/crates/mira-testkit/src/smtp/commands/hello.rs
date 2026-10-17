//! Commandes EHLO et HELO (RFC 5321 4.1.1.1).

use std::io;

use crate::smtp::session::{Flow, Session};

const EHLO_REPLY: &str = "250-127.0.0.1 MIRA testkit\r\n250-AUTH PLAIN LOGIN XOAUTH2\r\n250-SIZE 35882577\r\n250-8BITMIME\r\n250 PIPELINING";

/// Répond à EHLO en annonçant l'authentification et les extensions
pub(crate) async fn ehlo(session: &mut Session) -> io::Result<Flow> {
    session.transaction.reset();
    session.answer(EHLO_REPLY).await
}

/// Répond à HELO sans extension
pub(crate) async fn helo(session: &mut Session) -> io::Result<Flow> {
    session.transaction.reset();
    session.answer("250 127.0.0.1 MIRA testkit").await
}
