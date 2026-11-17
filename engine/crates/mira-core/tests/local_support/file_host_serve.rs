#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, dead_code)]
//! Traitement d'une connexion du faux hébergeur : en-têtes, redirection, `Range`, coupure.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use super::file_host::{Script, Seen};

/// Lit l'en-tête de la requête : ligne de départ et en-têtes en minuscules
async fn read_head(stream: &mut TcpStream) -> (String, HashMap<String, String>) {
    let mut buffer = Vec::new();
    let mut byte = [0u8; 1];
    while !buffer.ends_with(b"\r\n\r\n") {
        if stream.read(&mut byte).await.unwrap_or(0) == 0 {
            break;
        }
        buffer.push(byte[0]);
    }
    let text = String::from_utf8_lossy(&buffer).to_string();
    let mut lines = text.lines();
    let start = lines.next().unwrap_or_default().to_string();
    let headers = lines
        .filter_map(|l| l.split_once(':'))
        .map(|(k, v)| (k.trim().to_lowercase(), v.trim().to_string()))
        .collect();
    (start, headers)
}

/// Début demandé par `Range: bytes=N-`, 0 sans `Range`
fn range_start(range: Option<&String>) -> usize {
    range
        .and_then(|r| r.strip_prefix("bytes="))
        .and_then(|r| r.split('-').next())
        .and_then(|n| n.parse().ok())
        .unwrap_or(0)
}

/// Sert une connexion : `/file` redirige, `/blob` sert le contenu entier ou depuis `Range`
pub async fn serve(
    mut stream: TcpStream,
    content: Arc<Vec<u8>>,
    script: Arc<Mutex<Script>>,
    seen: Arc<Mutex<Vec<Seen>>>,
) {
    let (start, headers) = read_head(&mut stream).await;
    let path = start.split_whitespace().nth(1).unwrap_or("/").to_string();
    let range = headers.get("range").cloned();
    seen.lock().unwrap().push(Seen {
        path: path.clone(),
        range: range.clone(),
        has_authorization: headers.contains_key("authorization"),
    });
    if path == "/file" {
        let head = "HTTP/1.1 302 Found\r\nLocation: /blob\r\nContent-Length: 0\r\n\r\n";
        stream.write_all(head.as_bytes()).await.unwrap();
        return;
    }
    send_blob(&mut stream, &content, &script, range.as_ref()).await;
}

/// Lit le contenu de remplacement et consomme la coupure programmée
fn take_script(script: &Mutex<Script>) -> (Option<Vec<u8>>, Option<usize>) {
    let mut script = script.lock().unwrap();
    (
        script.other_content_on_range.clone(),
        script.cut_after.take(),
    )
}

/// Envoie le contenu (ou sa suite) ; coupe après N octets si le test l'a demandé une fois
async fn send_blob(
    stream: &mut TcpStream,
    content: &[u8],
    script: &Mutex<Script>,
    range: Option<&String>,
) {
    let from = range_start(range);
    let (other, cut) = take_script(script);
    let source = match (&other, from > 0) {
        (Some(other), true) => other.as_slice(),
        _ => content,
    };
    let body = &source[from.min(source.len())..];
    let status = if from > 0 {
        "206 Partial Content"
    } else {
        "200 OK"
    };
    let total = source.len();
    let head = format!(
        "HTTP/1.1 {status}\r\nContent-Length: {}\r\nContent-Range: bytes {from}-{}/{total}\r\nAccept-Ranges: bytes\r\n\r\n",
        body.len(),
        total.saturating_sub(1)
    );
    stream.write_all(head.as_bytes()).await.unwrap();
    let sent = cut.map_or(body.len(), |n| n.min(body.len()));
    let _ = stream.write_all(&body[..sent]).await;
    let _ = stream.shutdown().await;
}
