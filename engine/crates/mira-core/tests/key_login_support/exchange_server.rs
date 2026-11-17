//! Faux point d'échange de clé en boucle locale : garde les corps reçus, répond à la demande du test.

use std::sync::{Arc, Mutex};

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

/// Serveur d'échange : réponse fixée à l'ouverture, corps JSON reçus conservés.
pub struct ExchangeServer {
    pub url: String,
    pub bodies: Arc<Mutex<Vec<String>>>,
}

impl ExchangeServer {
    /// Ouvre le serveur sur `127.0.0.1:0` ; il répond toujours `status` avec `body`
    pub async fn start(status: u16, body: &str) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}/api/v1/auth/keys", listener.local_addr().unwrap());
        let bodies = Arc::new(Mutex::new(Vec::new()));
        tokio::spawn(serve(
            listener,
            status,
            body.to_string(),
            Arc::clone(&bodies),
        ));
        Self { url, bodies }
    }

    /// Corps JSON reçus jusqu'ici
    pub fn received(&self) -> Vec<String> {
        self.bodies.lock().unwrap().clone()
    }
}

/// Accepte les connexions, chacune dans sa tâche
async fn serve(listener: TcpListener, status: u16, body: String, bodies: Arc<Mutex<Vec<String>>>) {
    loop {
        let Ok((stream, _)) = listener.accept().await else {
            return;
        };
        tokio::spawn(answer(stream, status, body.clone(), Arc::clone(&bodies)));
    }
}

/// Lit la requête, garde son corps puis répond
async fn answer(mut stream: TcpStream, status: u16, body: String, bodies: Arc<Mutex<Vec<String>>>) {
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
    let reply = format!(
        "HTTP/1.1 {status} X\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    let _ = stream.write_all(reply.as_bytes()).await;
}

/// Dit si les en-têtes et le corps annoncé sont entièrement arrivés
fn request_complete(raw: &[u8]) -> bool {
    let text = String::from_utf8_lossy(raw);
    let Some((head, rest)) = text.split_once("\r\n\r\n") else {
        return false;
    };
    let length = head
        .lines()
        .find_map(|line| {
            line.to_lowercase()
                .strip_prefix("content-length:")
                .map(|v| v.trim().parse::<usize>().unwrap_or(0))
        })
        .unwrap_or(0);
    rest.len() >= length
}
