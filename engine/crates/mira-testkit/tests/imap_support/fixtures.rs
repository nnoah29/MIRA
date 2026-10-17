//! Serveur de test, courrier d'exemple et sessions préparées.

use mira_testkit::Credentials;
use mira_testkit::imap::{CapabilitySet, FakeImapServer, FakeMailStore};

use super::client::RawClient;

/// Courrier d'exemple avec un identifiant de message connu.
pub const MAIL: &[u8] = b"From: Alice <alice@example.org>\r\nTo: bob@example.org\r\nSubject: Hello\r\nMessage-ID: <abc@mira>\r\n\r\nBody text\r\n";

/// Serveur de test avec son magasin et ses identifiants.
#[derive(Debug)]
pub struct Env {
    pub server: FakeImapServer,
    pub store: FakeMailStore,
    pub credentials: Credentials,
}

/// Courrier d'exemple avec un sujet et un identifiant choisis
pub fn mail(subject: &str, message_id: &str) -> Vec<u8> {
    format!("From: Alice <alice@example.org>\r\nSubject: {subject}\r\nMessage-ID: <{message_id}>\r\n\r\nBody of {subject}\r\n").into_bytes()
}

/// Démarre un serveur pour l'utilisateur `alice` (mot de passe `secret`, jeton `tok-1`)
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

/// Connexion déjà authentifiée en tant qu'`alice`
pub async fn logged_in(env: &Env) -> RawClient {
    let mut client = RawClient::connect(&env.server.address()).await;
    assert!(client.command("LOGIN alice secret").await.is_ok());
    client
}

/// Connexion authentifiée avec INBOX sélectionnée
pub async fn selected(env: &Env) -> RawClient {
    let mut client = logged_in(env).await;
    assert!(client.command("SELECT INBOX").await.is_ok());
    client
}
