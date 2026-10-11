//! Client HTTP/1.1 brut : écrit des octets, lit la réponse jusqu'à la fermeture.

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

/// Réponse analysée : statut, texte des en-têtes et corps.
pub struct HttpReply {
    pub status: u16,
    pub headers: String,
    pub body: String,
}

impl HttpReply {
    /// Valeur du champ JSON de premier niveau, sous forme de texte
    pub fn json_field(&self, name: &str) -> String {
        let value: serde_json::Value = serde_json::from_str(&self.body).unwrap();
        match &value[name] {
            serde_json::Value::String(text) => text.clone(),
            other => other.to_string(),
        }
    }
}

/// Envoie des octets bruts puis lit la réponse complète
pub async fn raw_exchange(address: &str, request: &[u8]) -> HttpReply {
    let mut stream = TcpStream::connect(address).await.unwrap();
    stream.write_all(request).await.unwrap();
    let mut text = String::new();
    stream.read_to_string(&mut text).await.unwrap();
    parse_reply(&text)
}

/// Découpe une réponse HTTP en statut, en-têtes et corps
fn parse_reply(text: &str) -> HttpReply {
    let (head, body) = text.split_once("\r\n\r\n").unwrap();
    let status = head.split(' ').nth(1).unwrap().parse().unwrap();
    HttpReply {
        status,
        headers: head.to_string(),
        body: body.to_string(),
    }
}

/// Envoie un POST avec le type de contenu donné
pub async fn post_with_type(
    address: &str,
    path: &str,
    content_type: &str,
    body: &str,
) -> HttpReply {
    let request = format!(
        "POST {path} HTTP/1.1\r\nHost: {address}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\n\r\n{body}",
        body.len()
    );
    raw_exchange(address, request.as_bytes()).await
}

/// Envoie un formulaire `application/x-www-form-urlencoded` en POST
pub async fn post_form(address: &str, path: &str, body: &str) -> HttpReply {
    post_with_type(address, path, "application/x-www-form-urlencoded", body).await
}
