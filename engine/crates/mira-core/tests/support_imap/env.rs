//! Faux serveur IMAP démarré pour un test, avec l'utilisateur `alice`.

use mira_testkit::Credentials;
use mira_testkit::imap::{CapabilitySet, FakeImapServer, FakeMailStore};
use mira_vocabulary::{Security, ServerEndpoint};

/// Serveur de test avec son magasin et ses identifiants.
#[derive(Debug)]
pub struct Env {
    pub server: FakeImapServer,
    pub store: FakeMailStore,
    pub credentials: Credentials,
}

/// Démarre un serveur pour `alice` (mot de passe `secret`, jeton `tok-1`)
pub async fn start(caps: CapabilitySet) -> Env {
    let store = FakeMailStore::new();
    let credentials = Credentials::default();
    credentials.add_password("alice", "secret");
    credentials.add_access_token("alice", "tok-1");
    let server = FakeImapServer::start(store.clone(), credentials.clone(), caps)
        .await
        .unwrap();
    Env {
        server,
        store,
        credentials,
    }
}

/// Adresse du serveur de test, en clair sur la boucle locale
pub fn endpoint(env: &Env) -> ServerEndpoint {
    ServerEndpoint {
        host: "127.0.0.1".to_string(),
        port: env.server.port(),
        security: Security::Plaintext,
    }
}

/// Adresse en boucle locale sur un port et un mode de chiffrement choisis
pub fn local_endpoint(port: u16, security: Security) -> ServerEndpoint {
    ServerEndpoint {
        host: "127.0.0.1".to_string(),
        port,
        security,
    }
}
