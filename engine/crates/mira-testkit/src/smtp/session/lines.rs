//! Lecture et écriture de lignes sur la connexion d'une session.

use std::io;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt};

use super::{Flow, Session};

impl Session {
    /// Lit une ligne brute terminée par LF ; la fin de flux est une erreur
    pub(crate) async fn read_raw_line(&mut self) -> io::Result<Vec<u8>> {
        let mut line = Vec::new();
        if self.stream.read_until(b'\n', &mut line).await? == 0 {
            return Err(io::ErrorKind::UnexpectedEof.into());
        }
        Ok(line)
    }

    /// Lit une ligne de commande sans son CRLF final
    pub(crate) async fn read_line(&mut self) -> io::Result<String> {
        let raw = self.read_raw_line().await?;
        let text = String::from_utf8_lossy(&raw);
        Ok(text.trim_end_matches(['\r', '\n']).to_string())
    }

    /// Écrit une réponse terminée par CRLF
    pub(crate) async fn reply(&mut self, text: &str) -> io::Result<()> {
        let line = format!("{text}\r\n");
        self.stream.write_all(line.as_bytes()).await?;
        self.stream.flush().await
    }

    /// Écrit une réponse puis demande de continuer le dialogue
    pub(crate) async fn answer(&mut self, text: &str) -> io::Result<Flow> {
        self.reply(text).await?;
        Ok(Flow::Continue)
    }

    /// Retire la prochaine panne armée par les tests, s'il y en a une
    pub(crate) fn take_failure(&self) -> Option<String> {
        self.shared.with(|state| state.failures.pop_front())
    }
}
