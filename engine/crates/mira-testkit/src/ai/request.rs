//! Lecture bornée d'une requête HTTP/1.1 : ligne, en-têtes et corps selon `Content-Length`.

use std::io;

use tokio::io::{AsyncBufRead, AsyncBufReadExt, AsyncReadExt};

use super::recorded::RecordedRequest;

const MAX_LINE: u64 = 8192;
const MAX_HEADERS: usize = 64;
const MAX_BODY: usize = 8 * 1024 * 1024;

/// Lit une requête complète depuis le flux
pub(super) async fn read_request<R: AsyncBufRead + Unpin>(
    reader: &mut R,
) -> io::Result<RecordedRequest> {
    let line = read_line(reader).await?;
    let (method, path) = parse_request_line(&line)?;
    let headers = read_headers(reader).await?;
    let length = content_length(&headers)?;
    let mut body = vec![0; length];
    reader.read_exact(&mut body).await?;
    let body = String::from_utf8_lossy(&body).into_owned();
    Ok(RecordedRequest {
        method,
        path,
        headers,
        body,
    })
}

/// Lit une ligne terminée par LF et la renvoie sans son CRLF
async fn read_line<R: AsyncBufRead + Unpin>(reader: &mut R) -> io::Result<String> {
    let mut line = String::new();
    let read = (&mut *reader).take(MAX_LINE).read_line(&mut line).await?;
    if read == 0 || !line.ends_with('\n') {
        return Err(io::Error::from(io::ErrorKind::InvalidInput));
    }
    Ok(line.trim_end_matches(['\r', '\n']).to_string())
}

/// Extrait la méthode et la cible (requête incluse) de la première ligne
fn parse_request_line(line: &str) -> io::Result<(String, String)> {
    let mut fields = line.split(' ');
    match (fields.next(), fields.next(), fields.next(), fields.next()) {
        (Some(method), Some(target), Some(version), None) if version.starts_with("HTTP/1.") => {
            Ok((method.to_string(), target.to_string()))
        }
        _ => Err(io::Error::from(io::ErrorKind::InvalidInput)),
    }
}

/// Lit les en-têtes jusqu'à la ligne vide, noms en minuscules
async fn read_headers<R: AsyncBufRead + Unpin>(
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
            .ok_or_else(|| io::Error::from(io::ErrorKind::InvalidInput))?;
        headers.push((name.trim().to_ascii_lowercase(), value.trim().to_string()));
        if headers.len() > MAX_HEADERS {
            return Err(io::Error::from(io::ErrorKind::InvalidInput));
        }
    }
}

/// Taille déclarée du corps, zéro si l'en-tête est absent
fn content_length(headers: &[(String, String)]) -> io::Result<usize> {
    let Some((_, value)) = headers.iter().find(|(name, _)| name == "content-length") else {
        return Ok(0);
    };
    match value.parse::<usize>() {
        Ok(length) if length <= MAX_BODY => Ok(length),
        _ => Err(io::Error::from(io::ErrorKind::InvalidInput)),
    }
}
