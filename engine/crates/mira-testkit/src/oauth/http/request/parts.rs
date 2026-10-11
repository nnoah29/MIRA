//! Éléments de lecture d'une requête HTTP, avec des limites de taille.

use std::io;

use tokio::io::{AsyncBufRead, AsyncBufReadExt, AsyncReadExt};

use crate::oauth::error::invalid_input;

const MAX_LINE: u64 = 8192;
const MAX_HEADERS: usize = 64;
const MAX_BODY: usize = 65536;

/// Lit une ligne terminée par LF et la renvoie sans son CRLF
pub(super) async fn read_line<R: AsyncBufRead + Unpin>(reader: &mut R) -> io::Result<String> {
    let mut line = String::new();
    let read = (&mut *reader).take(MAX_LINE).read_line(&mut line).await?;
    if read == 0 || !line.ends_with('\n') {
        return Err(invalid_input("incomplete or oversized line"));
    }
    Ok(line.trim_end_matches(['\r', '\n']).to_string())
}

/// Extrait la méthode et le chemin (sans requête ni fragment) de la première ligne
pub(super) fn parse_request_line(line: &str) -> io::Result<(String, String)> {
    let mut fields = line.split(' ');
    let (Some(method), Some(target), Some(version), None) =
        (fields.next(), fields.next(), fields.next(), fields.next())
    else {
        return Err(invalid_input("malformed request line"));
    };
    if !version.starts_with("HTTP/1.") {
        return Err(invalid_input("unsupported HTTP version"));
    }
    let path = target.split(['?', '#']).next().unwrap_or(target);
    Ok((method.to_string(), path.to_string()))
}

/// Lit les en-têtes jusqu'à la ligne vide, noms en minuscules
pub(super) async fn read_headers<R: AsyncBufRead + Unpin>(
    reader: &mut R,
) -> io::Result<Vec<(String, String)>> {
    let mut headers = Vec::new();
    loop {
        let line = read_line(reader).await?;
        if line.is_empty() {
            return Ok(headers);
        }
        let (name, value) = line
            .split_once(':')
            .ok_or_else(|| invalid_input("malformed header"))?;
        headers.push((name.trim().to_ascii_lowercase(), value.trim().to_string()));
        if headers.len() > MAX_HEADERS {
            return Err(invalid_input("too many headers"));
        }
    }
}

/// Taille déclarée du corps, zéro si l'en-tête est absent
pub(super) fn content_length(headers: &[(String, String)]) -> io::Result<usize> {
    let Some((_, value)) = headers.iter().find(|(name, _)| name == "content-length") else {
        return Ok(0);
    };
    let length: usize = value
        .parse()
        .map_err(|_| invalid_input("invalid content-length"))?;
    if length > MAX_BODY {
        return Err(invalid_input("body too large"));
    }
    Ok(length)
}

/// Lit exactement `length` octets de corps
pub(super) async fn read_body<R: AsyncBufRead + Unpin>(
    reader: &mut R,
    length: usize,
) -> io::Result<Vec<u8>> {
    let mut body = vec![0; length];
    reader.read_exact(&mut body).await?;
    Ok(body)
}
