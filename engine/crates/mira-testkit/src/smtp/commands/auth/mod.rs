//! Commande AUTH (RFC 4954) : PLAIN, LOGIN et XOAUTH2.

mod login;
mod plain;
mod xoauth2;

use std::io;

use crate::smtp::session::{Flow, Session};

/// Authentifie le client avec le mécanisme demandé
pub(crate) async fn handle(session: &mut Session, args: &str) -> io::Result<Flow> {
    if session.transaction.user.is_some() {
        return session.answer("503 5.5.1 Already authenticated").await;
    }
    let (mechanism, initial) = split_mechanism(args);
    let user = match mechanism.as_str() {
        "PLAIN" => plain::authenticate(session, initial).await?,
        "LOGIN" => login::authenticate(session, initial).await?,
        "XOAUTH2" => xoauth2::authenticate(session, initial).await?,
        _ => {
            return session
                .answer("504 5.5.4 Unrecognized authentication type")
                .await;
        }
    };
    complete(session, user).await
}

/// Sépare le nom du mécanisme de la réponse initiale facultative
fn split_mechanism(args: &str) -> (String, Option<&str>) {
    match args.split_once(' ') {
        Some((name, initial)) => (name.to_ascii_uppercase(), Some(initial.trim())),
        None => (args.to_ascii_uppercase(), None),
    }
}

/// Renvoie la réponse initiale du client ou la demande par un défi 334
pub(super) async fn obtain_response(
    session: &mut Session,
    initial: Option<&str>,
    prompt: &str,
) -> io::Result<String> {
    if let Some(response) = initial {
        return Ok(response.to_string());
    }
    session.reply(&format!("334 {prompt}")).await?;
    session.read_line().await
}

/// Clôt l'échange : 235 et mémorisation de l'utilisateur, ou 535
async fn complete(session: &mut Session, user: Option<String>) -> io::Result<Flow> {
    let Some(user) = user else {
        return session
            .answer("535 5.7.8 Authentication credentials invalid")
            .await;
    };
    session.transaction.user = Some(user);
    session.answer("235 2.7.0 Authentication successful").await
}
