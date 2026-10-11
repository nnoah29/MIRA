//! Scénarios réutilisables : démarrage, authentification, envoi d'un message.

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use mira_testkit::Credentials;
use mira_testkit::smtp::FakeSmtpServer;

use super::client::SmtpClient;

/// Encode une réponse AUTH PLAIN
pub fn plain_b64(user: &str, password: &str) -> String {
    STANDARD.encode(format!("\0{user}\0{password}"))
}

/// Encode une réponse AUTH XOAUTH2
pub fn xoauth2_b64(user: &str, token: &str) -> String {
    STANDARD.encode(format!("user={user}\u{1}auth=Bearer {token}\u{1}\u{1}"))
}

/// Démarre un serveur qui connaît alice avec le mot de passe `secret`
pub async fn started_server() -> (FakeSmtpServer, Credentials) {
    let credentials = Credentials::default();
    credentials.add_password("alice", "secret");
    let server = FakeSmtpServer::start(credentials.clone()).await.unwrap();
    (server, credentials)
}

/// Ouvre une connexion, annonce EHLO et s'authentifie en PLAIN comme alice
pub async fn authenticated_client(server: &FakeSmtpServer) -> SmtpClient {
    let mut client = SmtpClient::connect(&server.address()).await;
    assert!(client.command("EHLO client.test").await.starts_with("250"));
    let reply = client
        .command(&format!("AUTH PLAIN {}", plain_b64("alice", "secret")))
        .await;
    assert!(reply.starts_with("235"), "authentification : {reply}");
    client
}

/// Envoie un message complet avec bourrage des points et renvoie la réponse finale
pub async fn send_message(
    client: &mut SmtpClient,
    from: &str,
    recipients: &[&str],
    body: &str,
) -> String {
    assert!(
        client
            .command(&format!("MAIL FROM:<{from}>"))
            .await
            .starts_with("250")
    );
    for recipient in recipients {
        let reply = client.command(&format!("RCPT TO:<{recipient}>")).await;
        assert!(reply.starts_with("250"), "RCPT : {reply}");
    }
    assert!(client.command("DATA").await.starts_with("354"));
    let stuffed: String = body
        .split_inclusive("\r\n")
        .map(|line| {
            if line.starts_with('.') {
                format!(".{line}")
            } else {
                line.to_string()
            }
        })
        .collect();
    client.send_raw(format!("{stuffed}.\r\n").as_bytes()).await;
    client.read_reply().await
}
