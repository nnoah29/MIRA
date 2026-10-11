#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, dead_code)]
//! Fabriques partagées par les tests d'authentification : faux service OAuth, aléa, navigateur.

pub mod browser;

use std::time::Duration;

use mira_testkit::Credentials;
use mira_testkit::oauth::FakeOAuthServer;
use mira_vocabulary::OAuthSettings;
use rand::SeedableRng;
use rand::rngs::StdRng;

pub const LIMIT: Duration = Duration::from_secs(20);
pub const CLIENT_ID: &str = "mira-test-client";
pub const USER: &str = "alice@example.org";

/// Démarre un faux serveur OAuth avec un registre d'identifiants vide
pub async fn started_oauth() -> (FakeOAuthServer, Credentials) {
    let credentials = Credentials::default();
    let server = FakeOAuthServer::start(credentials.clone()).await.unwrap();
    (server, credentials)
}

/// Paramètres OAuth qui pointent vers le faux serveur
pub fn oauth_settings(server: &FakeOAuthServer) -> OAuthSettings {
    OAuthSettings {
        authorize_url: server.authorize_url(),
        token_url: server.token_url(),
        scopes: vec!["mail".to_string(), "imap smtp".to_string()],
    }
}

/// Générateur aléatoire reproductible pour les tests
pub fn seeded_rng(seed: u64) -> StdRng {
    StdRng::seed_from_u64(seed)
}
