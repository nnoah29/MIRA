//! Faux point de jeton en boucle locale : réponses mises en file par le test, formulaires reçus conservés.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use super::http_read::request_complete;

type Replies = Arc<Mutex<VecDeque<(u16, String)>>>;

/// Serveur de jetons ; la dernière réponse mise en file est répétée tant qu'aucune autre n'arrive.
pub struct TokenServer {
    pub url: String,
    pub bodies: Arc<Mutex<Vec<String>>>,
    replies: Replies,
}

impl TokenServer {
    /// Ouvre le serveur sur `127.0.0.1:0` ; sans réponse en file il rend 500
    pub async fn start() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}/oauth/token", listener.local_addr().unwrap());
        let bodies = Arc::new(Mutex::new(Vec::new()));
        let replies: Replies = Arc::new(Mutex::new(VecDeque::new()));
        tokio::spawn(serve(listener, Arc::clone(&bodies), Arc::clone(&replies)));
        Self {
            url,
            bodies,
            replies,
        }
    }

    /// Met une réponse en file
    pub fn push(&self, status: u16, body: &str) {
        self.replies
            .lock()
            .unwrap()
            .push_back((status, body.to_string()));
    }

    /// Formulaires reçus jusqu'ici
    pub fn received(&self) -> Vec<String> {
        self.bodies.lock().unwrap().clone()
    }
}

/// Accepte les connexions, chacune dans sa tâche
async fn serve(listener: TcpListener, bodies: Arc<Mutex<Vec<String>>>, replies: Replies) {
    loop {
        let Ok((stream, _)) = listener.accept().await else {
            return;
        };
        tokio::spawn(answer(stream, Arc::clone(&bodies), Arc::clone(&replies)));
    }
}

/// Lit la requête, garde son corps puis rend la prochaine réponse (la dernière reste)
async fn answer(mut stream: TcpStream, bodies: Arc<Mutex<Vec<String>>>, replies: Replies) {
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
    let reply = format!(
        "HTTP/1.1 {status} X\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    let _ = stream.write_all(reply.as_bytes()).await;
}

/// Prochaine réponse ; la dernière est conservée pour les appels suivants
fn next_reply(replies: &Replies) -> (u16, String) {
    let mut queue = replies.lock().unwrap();
    if queue.len() > 1 {
        return queue.pop_front().unwrap();
    }
    queue.front().cloned().unwrap_or((500, "{}".to_string()))
}
