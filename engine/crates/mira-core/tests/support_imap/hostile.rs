//! Serveur IMAP hostile : authentifie le client puis répond à `UID FETCH` par un scénario d'octets choisi.

use std::time::Duration;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;
use tokio::task::JoinHandle;

/// Une étape du scénario servi en réponse au `UID FETCH`.
#[derive(Debug, Clone)]
pub enum Step {
    Send(Vec<u8>),
    Pause(Duration),
}

/// Remplace `{tag}` par l'étiquette de la commande reçue
fn with_tag(bytes: &[u8], tag: &str) -> Vec<u8> {
    let text = String::from_utf8_lossy(bytes).replace("{tag}", tag);
    text.into_bytes()
}

/// Joue le scénario sur le flux d'écriture
async fn play(write: &mut tokio::net::tcp::OwnedWriteHalf, tag: &str, steps: &[Step]) {
    for step in steps {
        match step {
            Step::Send(bytes) => {
                if write.write_all(&with_tag(bytes, tag)).await.is_err() {
                    return;
                }
            }
            Step::Pause(delay) => tokio::time::sleep(*delay).await,
        }
    }
}

/// Sert une connexion : bannière, LOGIN et SELECT acceptés, scénario pour `UID FETCH`
async fn serve(listener: TcpListener, steps: Vec<Step>) {
    let Ok((stream, _)) = listener.accept().await else {
        return;
    };
    let (read, mut write) = stream.into_split();
    let mut reader = BufReader::new(read);
    let _ = write
        .write_all(b"* OK [CAPABILITY IMAP4rev1] pret\r\n")
        .await;
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line).await.unwrap_or(0) == 0 {
            return;
        }
        let tag = line.split_whitespace().next().unwrap_or("*").to_string();
        if line.contains("UID FETCH") {
            play(&mut write, &tag, &steps).await;
        } else {
            let _ = write
                .write_all(format!("{tag} OK [CAPABILITY IMAP4rev1] fait\r\n").as_bytes())
                .await;
        }
    }
}

/// Lance le serveur hostile sur un port éphémère de la boucle locale
pub async fn hostile_server(steps: Vec<Step>) -> (u16, JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    (port, tokio::spawn(serve(listener, steps)))
}
