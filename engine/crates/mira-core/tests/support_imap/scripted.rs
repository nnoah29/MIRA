//! Petit serveur scripté pour les cas que le faux serveur ne sait pas jouer (mécanismes absents, silence, TLS).

use std::time::Duration;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;
use tokio::task::JoinHandle;

/// Sert une connexion : envoie la bannière, puis répond dans l'ordre à chaque ligne reçue.
async fn serve_script(
    listener: TcpListener,
    greeting: String,
    replies: Vec<String>,
) -> Vec<String> {
    let (stream, _) = listener.accept().await.unwrap();
    let (read, mut write) = stream.into_split();
    let mut reader = BufReader::new(read);
    write.write_all(greeting.as_bytes()).await.unwrap();
    let mut received = Vec::new();
    for reply in replies {
        let mut line = String::new();
        if reader.read_line(&mut line).await.unwrap_or(0) == 0 {
            break;
        }
        let tag = line.split_whitespace().next().unwrap_or("*").to_string();
        received.push(line.trim_end().to_string());
        write
            .write_all(reply.replace("{tag}", &tag).as_bytes())
            .await
            .unwrap();
    }
    tokio::time::sleep(Duration::from_secs(1)).await;
    received
}

/// Lance un serveur scripté ; `{tag}` dans une réponse est remplacé par l'étiquette de la commande reçue.
pub async fn scripted(greeting: &str, replies: &[&str]) -> (u16, JoinHandle<Vec<String>>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let replies = replies.iter().map(|reply| reply.to_string()).collect();
    let task = tokio::spawn(serve_script(listener, greeting.to_string(), replies));
    (port, task)
}
