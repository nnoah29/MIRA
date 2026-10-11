//! Client TCP brut : écrit une commande, lit la réponse SMTP.

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;

/// Connexion cliente brute vers un faux serveur SMTP.
pub struct SmtpClient {
    reader: BufReader<TcpStream>,
}

impl SmtpClient {
    /// Se connecte et vérifie la bannière 220
    pub async fn connect(address: &str) -> Self {
        let stream = TcpStream::connect(address).await.unwrap();
        let mut client = Self {
            reader: BufReader::new(stream),
        };
        let banner = client.read_reply().await;
        assert!(banner.starts_with("220"), "bannière inattendue : {banner}");
        client
    }

    /// Écrit des octets tels quels
    pub async fn send_raw(&mut self, bytes: &[u8]) {
        self.reader.write_all(bytes).await.unwrap();
        self.reader.flush().await.unwrap();
    }

    /// Lit une réponse complète, lignes de continuation comprises, séparées par LF
    pub async fn read_reply(&mut self) -> String {
        let mut lines = Vec::new();
        loop {
            let mut line = String::new();
            let read = self.reader.read_line(&mut line).await.unwrap();
            let line = line.trim_end().to_string();
            let last = line.as_bytes().get(3) != Some(&b'-');
            if read > 0 {
                lines.push(line);
            }
            if read == 0 || last {
                return lines.join("\n");
            }
        }
    }

    /// Envoie une ligne de commande et renvoie la réponse
    pub async fn command(&mut self, line: &str) -> String {
        self.send_raw(format!("{line}\r\n").as_bytes()).await;
        self.read_reply().await
    }

    /// Indique si le serveur a fermé la connexion sans rien envoyer de plus
    pub async fn is_closed_by_peer(&mut self) -> bool {
        let mut line = String::new();
        match self.reader.read_line(&mut line).await {
            Ok(read) => read == 0,
            Err(_) => true,
        }
    }
}
