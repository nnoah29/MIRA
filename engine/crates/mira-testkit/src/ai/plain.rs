//! Réponses HTTP/1.1 complètes avec `Content-Length`, fermées après l'envoi.

use std::io;

use tokio::io::{AsyncWrite, AsyncWriteExt};

/// Écrit une réponse complète ; `application/json` si aucun type de contenu n'est donné
pub(super) async fn write_plain<W: AsyncWrite + Unpin>(
    writer: &mut W,
    code: u16,
    headers: &[(String, String)],
    body: &str,
) -> io::Result<()> {
    let mut head = format!("HTTP/1.1 {code} {}\r\n", reason(code));
    for (name, value) in headers {
        head.push_str(&format!("{name}: {value}\r\n"));
    }
    if !headers
        .iter()
        .any(|(n, _)| n.eq_ignore_ascii_case("content-type"))
    {
        head.push_str("Content-Type: application/json\r\n");
    }
    head.push_str(&format!(
        "Content-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    ));
    writer.write_all(head.as_bytes()).await?;
    writer.write_all(body.as_bytes()).await?;
    writer.flush().await
}

/// Phrase de statut usuelle pour le code, « Status » sinon
fn reason(code: u16) -> &'static str {
    match code {
        200 => "OK",
        302 => "Found",
        400 => "Bad Request",
        401 => "Unauthorized",
        403 => "Forbidden",
        404 => "Not Found",
        429 => "Too Many Requests",
        500 => "Internal Server Error",
        502 => "Bad Gateway",
        503 => "Service Unavailable",
        _ => "Status",
    }
}
