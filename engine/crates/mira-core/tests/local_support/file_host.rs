#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, dead_code)]
//! Faux hébergeur de fichiers sur 127.0.0.1 : redirection, reprise par `Range`, coupures programmées.

use std::sync::{Arc, Mutex};

use tokio::net::TcpListener;

use super::file_host_serve::serve;

/// Ce que le serveur a vu d'une requête
#[derive(Debug, Clone)]
pub struct Seen {
    pub path: String,
    pub range: Option<String>,
    pub has_authorization: bool,
}

/// Comportements programmés à la volée par le test
#[derive(Default)]
pub struct Script {
    pub cut_after: Option<usize>,
    pub other_content_on_range: Option<Vec<u8>>,
}

/// Faux hébergeur : `/file` redirige (302) vers `/blob`, qui sert le contenu avec `Range`
pub struct FileHost {
    port: u16,
    script: Arc<Mutex<Script>>,
    seen: Arc<Mutex<Vec<Seen>>>,
}

impl FileHost {
    /// Démarre le serveur sur un port libre de la boucle locale
    pub async fn start(content: Vec<u8>) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let script = Arc::new(Mutex::new(Script::default()));
        let seen = Arc::new(Mutex::new(Vec::new()));
        let (content, s, v) = (Arc::new(content), script.clone(), seen.clone());
        tokio::spawn(async move {
            while let Ok((stream, _)) = listener.accept().await {
                tokio::spawn(serve(stream, content.clone(), s.clone(), v.clone()));
            }
        });
        Self { port, script, seen }
    }

    /// Adresse du fichier (celle qui redirige)
    pub fn url(&self) -> String {
        format!("http://127.0.0.1:{}/file", self.port)
    }

    /// La prochaine réponse de contenu s'arrête après `bytes` octets de corps
    pub fn cut_next_response_after(&self, bytes: usize) {
        self.script.lock().unwrap().cut_after = Some(bytes);
    }

    /// Les reprises (`Range`) reçoivent un autre contenu, comme un fichier changé en amont
    pub fn serve_other_content_on_range(&self, other: Vec<u8>) {
        self.script.lock().unwrap().other_content_on_range = Some(other);
    }

    /// En-tête `Range` de la dernière requête de contenu
    pub fn last_blob_range(&self) -> Option<String> {
        let seen = self.seen.lock().unwrap();
        seen.iter().rev().find(|r| r.path == "/blob")?.range.clone()
    }

    /// Requêtes reçues jusqu'ici
    pub fn requests(&self) -> Vec<Seen> {
        self.seen.lock().unwrap().clone()
    }
}
