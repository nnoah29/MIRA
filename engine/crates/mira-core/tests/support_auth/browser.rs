//! Simulation du navigateur : requêtes brutes vers l'écouteur de redirection du moteur.

use mira_core::auth::form::parse_query;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

/// Envoie `GET <cible>` à l'adresse de redirection et renvoie la réponse complète
pub async fn get(redirect_uri: &str, target: &str) -> String {
    let authority = redirect_uri
        .strip_prefix("http://")
        .and_then(|rest| rest.split('/').next())
        .unwrap();
    let mut stream = TcpStream::connect(authority).await.unwrap();
    let request =
        format!("GET {target} HTTP/1.1\r\nHost: {authority}\r\nConnection: close\r\n\r\n");
    stream.write_all(request.as_bytes()).await.unwrap();
    let mut reply = Vec::new();
    stream.read_to_end(&mut reply).await.unwrap();
    String::from_utf8_lossy(&reply).into_owned()
}

/// Valeur décodée d'un paramètre de la requête d'une URL
pub fn query_value(url: &str, name: &str) -> Option<String> {
    let (_, query) = url.split_once('?')?;
    let pairs = parse_query(query);
    pairs
        .into_iter()
        .find(|(key, _)| key == name)
        .map(|(_, value)| value)
}
