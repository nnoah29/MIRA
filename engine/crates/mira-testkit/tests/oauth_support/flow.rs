//! Scénario d'autorisation : démarrage, URL d'autorisation et approbation.

use mira_testkit::Credentials;
use mira_testkit::oauth::FakeOAuthServer;
use mira_testkit::oauth::form::{Form, build_query};
use mira_testkit::oauth::pkce;

use super::loopback::Loopback;

pub const CLIENT_ID: &str = "mira-test-client";
pub const VERIFIER: &str = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";

/// Code et état reçus par la redirection, avec l'adresse de redirection utilisée.
pub struct Grant {
    pub code: String,
    pub state: String,
    pub redirect_uri: String,
}

/// Démarre un serveur OAuth avec un registre d'identifiants vide
pub async fn started() -> (FakeOAuthServer, Credentials) {
    let credentials = Credentials::default();
    let server = FakeOAuthServer::start(credentials.clone()).await.unwrap();
    (server, credentials)
}

/// Construit l'URL d'autorisation complète avec PKCE S256
pub fn authorize_url(server: &FakeOAuthServer, redirect_uri: &str, state: &str) -> String {
    let challenge = pkce::challenge_for(VERIFIER);
    let query = build_query(&[
        ("response_type", "code"),
        ("client_id", CLIENT_ID),
        ("redirect_uri", redirect_uri),
        ("state", state),
        ("code_challenge", &challenge),
        ("code_challenge_method", "S256"),
        ("scope", "mail"),
    ]);
    format!("{}?{query}", server.authorize_url())
}

/// Approuve pour l'utilisateur et renvoie ce que le moteur reçoit sur sa boucle locale
pub async fn authorize(server: &FakeOAuthServer, user: &str, state: &str) -> Grant {
    let loopback = Loopback::bind().await;
    let redirect_uri = loopback.redirect_uri();
    let url = authorize_url(server, &redirect_uri, state);
    let pending = loopback.expect_redirect();
    server.approve(&url, user).await.unwrap();
    let target = pending.await.unwrap();
    let form = Form::parse(target.split_once('?').unwrap().1);
    Grant {
        code: form.get("code").unwrap().to_string(),
        state: form.get("state").unwrap().to_string(),
        redirect_uri,
    }
}
