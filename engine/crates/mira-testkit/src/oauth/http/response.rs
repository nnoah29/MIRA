//! Réponse HTTP/1.1 JSON avec `Content-Length` et `Connection: close`.

use std::io;

use serde_json::{Value, json};
use tokio::io::{AsyncWrite, AsyncWriteExt};

/// Réponse prête à être écrite : statut et corps JSON.
#[derive(Debug, Clone)]
pub(crate) struct HttpResponse {
    status: u16,
    body: String,
}

impl HttpResponse {
    /// Réponse JSON avec le statut donné
    pub(crate) fn json(status: u16, value: &Value) -> Self {
        Self {
            status,
            body: value.to_string(),
        }
    }

    /// Réponse d'erreur OAuth `{"error": code}`
    pub(crate) fn error(status: u16, code: &str) -> Self {
        Self::json(status, &json!({ "error": code }))
    }

    /// Phrase de raison associée au statut
    fn reason(&self) -> &'static str {
        match self.status {
            200 => "OK",
            400 => "Bad Request",
            404 => "Not Found",
            405 => "Method Not Allowed",
            _ => "Error",
        }
    }

    /// Sérialise la réponse complète en octets
    fn to_bytes(&self) -> Vec<u8> {
        format!(
            "HTTP/1.1 {} {}\r\nContent-Type: application/json\r\nCache-Control: no-store\r\nPragma: no-cache\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            self.status,
            self.reason(),
            self.body.len(),
            self.body
        )
        .into_bytes()
    }

    /// Écrit la réponse sur le flux puis le vide
    pub(crate) async fn write_to<W: AsyncWrite + Unpin>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(&self.to_bytes()).await?;
        writer.flush().await
    }
}
