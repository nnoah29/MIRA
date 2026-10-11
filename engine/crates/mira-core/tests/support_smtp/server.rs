//! Faux serveur SMTP du testkit, adresse du serveur et identifiants de test.

use mira_core::ports::SecretValue;
use mira_core::smtp::compose::build_message;
use mira_core::smtp::{OutgoingMessage, SmtpCredential, SmtpLogin};
use mira_testkit::Credentials;
use mira_testkit::smtp::FakeSmtpServer;
use mira_vocabulary::{Security, ServerEndpoint};

use super::sample_input;

pub const USER: &str = "alice@example.org";
pub const PASSWORD: &str = "pw-Sup3r-Secret-42";

/// Démarre un faux serveur qui accepte le mot de passe de test
pub async fn started() -> (FakeSmtpServer, Credentials) {
    let credentials = Credentials::default();
    credentials.add_password(USER, PASSWORD);
    let server = FakeSmtpServer::start(credentials.clone()).await.unwrap();
    (server, credentials)
}

/// Point d'accès en clair du faux serveur, sur la boucle locale
pub fn endpoint(server: &FakeSmtpServer) -> ServerEndpoint {
    ServerEndpoint {
        host: "127.0.0.1".to_string(),
        port: server.port(),
        security: Security::Plaintext,
    }
}

/// Connexion par mot de passe ; `password` permet de tester un refus
pub fn password_login(password: &str) -> SmtpLogin {
    SmtpLogin {
        user: USER.to_string(),
        credential: SmtpCredential::Password(SecretValue::new(password)),
    }
}

/// Message d'exemple composé puis converti pour le client SMTP
pub fn outgoing() -> OutgoingMessage {
    build_message(&sample_input()).unwrap().into_outgoing()
}
