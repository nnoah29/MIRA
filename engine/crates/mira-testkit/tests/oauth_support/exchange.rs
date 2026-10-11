//! Requêtes vers `/token` : échange de code et rafraîchissement.

use mira_testkit::oauth::FakeOAuthServer;
use mira_testkit::oauth::form::build_query;

use super::flow::{CLIENT_ID, Grant, VERIFIER};
use super::http::{HttpReply, post_form};

/// Adresse d'écoute HTTP du serveur
pub fn address(server: &FakeOAuthServer) -> String {
    format!("127.0.0.1:{}", server.port())
}

/// Échange un code d'autorisation avec le vérificateur donné
pub async fn exchange_with(server: &FakeOAuthServer, grant: &Grant, verifier: &str) -> HttpReply {
    let body = build_query(&[
        ("grant_type", "authorization_code"),
        ("code", &grant.code),
        ("redirect_uri", &grant.redirect_uri),
        ("client_id", CLIENT_ID),
        ("code_verifier", verifier),
    ]);
    post_form(&address(server), "/token", &body).await
}

/// Échange un code d'autorisation avec le bon vérificateur
pub async fn exchange(server: &FakeOAuthServer, grant: &Grant) -> HttpReply {
    exchange_with(server, grant, VERIFIER).await
}

/// Demande un nouveau jeton d'accès avec un jeton de rafraîchissement
pub async fn refresh(server: &FakeOAuthServer, refresh_token: &str) -> HttpReply {
    let body = build_query(&[
        ("grant_type", "refresh_token"),
        ("refresh_token", refresh_token),
        ("client_id", CLIENT_ID),
    ]);
    post_form(&address(server), "/token", &body).await
}
