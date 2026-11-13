//! Faux point de jeton en boucle locale : réponses mises en file par le test, formulaires reçus conservés.

use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use super::http_read::request_complete;
use super::replies::{Replies, next_reply};

type Shared = (Arc<Mutex<Vec<String>>>, Replies, Arc<AtomicU64>);

/// Serveur de jetons ; la dernière réponse servie est répétée tant qu'aucune autre n'arrive.
pub struct TokenServer {
    pub url: String,
    pub bodies: Arc<Mutex<Vec<String>>>,
    replies: Replies,
    delay_ms: Arc<AtomicU64>,
}

impl TokenServer {
    /// Ouvre le serveur sur `127.0.0.1:0` ; sans réponse en file il rend 500
    pub async fn start() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}/oauth/token", listener.local_addr().unwrap());
        let bodies = Arc::new(Mutex::new(Vec::new()));
        let replies: Replies = Arc::new(Mutex::new((VecDeque::new(), None)));
        let delay_ms = Arc::new(AtomicU64::new(0));
        let shared = (
            Arc::clone(&bodies),
            Arc::clone(&replies),
            Arc::clone(&delay_ms),
        );
        tokio::spawn(serve(listener, shared));
        Self {
            url,
            bodies,
            replies,
            delay_ms,
        }
    }

    /// Met une réponse en file
    pub fn push(&self, status: u16, body: &str) {
        self.replies
            .lock()
            .unwrap()
            .0
            .push_back((status, body.to_string()));
    }

    /// Retarde chaque réponse de ce nombre de millisecondes
    pub fn set_delay(&self, millis: u64) {
        self.delay_ms.store(millis, Ordering::SeqCst);
    }

    /// Formulaires reçus jusqu'ici
    pub fn received(&self) -> Vec<String> {
        self.bodies.lock().unwrap().clone()
    }
}

/// Accepte les connexions, chacune dans sa tâche
async fn serve(listener: TcpListener, shared: Shared) {
    loop {
        let Ok((stream, _)) = listener.accept().await else {
            return;
        };
        let (bodies, replies, delay) = &shared;
        let own = (Arc::clone(bodies), Arc::clone(replies), Arc::clone(delay));
        tokio::spawn(answer(stream, own));
    }
}

/// Lit la requête, garde son corps puis rend la prochaine réponse (la dernière reste)
async fn answer(mut stream: TcpStream, shared: Shared) {
    let (bodies, replies, delay) = shared;
    let mut raw = Vec::new();
    let mut chunk = [0_u8; 2048];
    while !request_complete(&raw) {
        match stream.read(&mut chunk).await {
            Ok(0) | Err(_) => break,
            Ok(count) => raw.extend_from_slice(&chunk[..count]),
        }
    }
    let text = String::from_utf8_lossy(&raw).to_string();
    bodies
        .lock()
        .unwrap()
        .push(text.split("\r\n\r\n").nth(1).unwrap_or("").to_string());
    let (status, body) = next_reply(&replies);
    tokio::time::sleep(std::time::Duration::from_millis(
        delay.load(Ordering::SeqCst),
    ))
    .await;
    let reply = format!(
        "HTTP/1.1 {status} X\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    let _ = stream.write_all(reply.as_bytes()).await;
}
