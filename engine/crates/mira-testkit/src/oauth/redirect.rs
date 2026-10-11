//! Redirection du « navigateur » : requête GET vers l'écouteur de boucle locale du moteur.

use std::io;
use std::time::Duration;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::timeout;

use super::error::invalid_input;
use super::form::build_query;

const RESPONSE_WAIT: Duration = Duration::from_secs(5);
const LOOPBACK_ONLY: &str = "redirect_uri must be http://<loopback>:<port>/...";

/// Cible de redirection : autorité `hôte:port` et chemin avec requête éventuelle.
struct Target {
    authority: String,
    path: String,
}

/// Envoie `GET redirect_uri?params` et attend la réponse du moteur
pub(crate) async fn deliver(redirect_uri: &str, params: &[(&str, &str)]) -> io::Result<()> {
    let target = Target::parse(redirect_uri)?;
    let request = target.request_text(&build_query(params));
    let mut stream = TcpStream::connect(&target.authority).await?;
    stream.write_all(request.as_bytes()).await?;
    let mut response = Vec::new();
    let _ = timeout(RESPONSE_WAIT, stream.read_to_end(&mut response)).await;
    Ok(())
}

impl Target {
    /// Analyse un `redirect_uri` http dont l'hôte est une adresse de boucle locale
    fn parse(uri: &str) -> io::Result<Self> {
        let rest = uri
            .strip_prefix("http://")
            .ok_or_else(|| invalid_input(LOOPBACK_ONLY))?;
        let (authority, path) = rest.split_at(rest.find('/').unwrap_or(rest.len()));
        let path = if path.is_empty() { "/" } else { path };
        let (host, port) = authority
            .rsplit_once(':')
            .ok_or_else(|| invalid_input(LOOPBACK_ONLY))?;
        if !matches!(host, "127.0.0.1" | "localhost" | "[::1]") || port.parse::<u16>().is_err() {
            return Err(invalid_input(LOOPBACK_ONLY));
        }
        Ok(Self {
            authority: authority.to_string(),
            path: path.split('#').next().unwrap_or(path).to_string(),
        })
    }

    /// Texte de la requête GET avec les paramètres ajoutés à la requête existante
    fn request_text(&self, query: &str) -> String {
        let separator = if self.path.contains('?') { '&' } else { '?' };
        format!(
            "GET {}{separator}{query} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n",
            self.path, self.authority
        )
    }
}
