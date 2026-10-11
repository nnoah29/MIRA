//! Requête HTTP/1.1 : ligne de requête, en-têtes et corps selon `Content-Length`.

mod parts;

use std::io;

use tokio::io::AsyncBufRead;

/// Requête reçue : méthode, chemin sans requête, en-têtes (noms en minuscules) et corps.
#[derive(Debug)]
pub(crate) struct HttpRequest {
    pub(crate) method: String,
    pub(crate) path: String,
    pub(crate) headers: Vec<(String, String)>,
    pub(crate) body: Vec<u8>,
}

impl HttpRequest {
    /// Valeur du premier en-tête portant ce nom, sans tenir compte de la casse
    pub(crate) fn header(&self, name: &str) -> Option<&str> {
        let found = self
            .headers
            .iter()
            .find(|(key, _)| key.eq_ignore_ascii_case(name));
        found.map(|(_, value)| value.as_str())
    }
}

/// Lit une requête complète depuis le flux
pub(crate) async fn read_request<R: AsyncBufRead + Unpin>(
    reader: &mut R,
) -> io::Result<HttpRequest> {
    let request_line = parts::read_line(reader).await?;
    let (method, path) = parts::parse_request_line(&request_line)?;
    let headers = parts::read_headers(reader).await?;
    let length = parts::content_length(&headers)?;
    let body = parts::read_body(reader, length).await?;
    Ok(HttpRequest {
        method,
        path,
        headers,
        body,
    })
}
