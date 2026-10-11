//! Lecture des lignes et des réponses, littéraux compris.

use tokio::io::{AsyncBufReadExt, AsyncReadExt};

use super::client::RawClient;
use super::response::Response;

/// Taille d'un littéral annoncé en fin de ligne
fn literal_size(line: &str) -> Option<usize> {
    let inner = line.trim_end().strip_suffix('}')?;
    inner.get(inner.rfind('{')? + 1..)?.parse().ok()
}

impl RawClient {
    /// Lit une ligne physique, sans sa fin de ligne
    async fn read_physical(&mut self) -> Option<String> {
        let mut buffer = Vec::new();
        let read = self.reader.read_until(b'\n', &mut buffer).await.ok()?;
        if read == 0 {
            return None;
        }
        Some(
            String::from_utf8_lossy(&buffer)
                .trim_end_matches(['\r', '\n'])
                .to_string(),
        )
    }

    /// Lit une ligne logique : les littéraux sont insérés à leur place
    pub async fn read_line(&mut self) -> Option<String> {
        let mut line = self.read_physical().await?;
        let mut last = line.clone();
        while let Some(size) = literal_size(&last) {
            let mut data = vec![0; size];
            self.reader.read_exact(&mut data).await.ok()?;
            line.push_str("\r\n");
            line.push_str(&String::from_utf8_lossy(&data));
            last = self.read_physical().await?;
            line.push_str(&last);
        }
        Some(line)
    }

    /// Lit jusqu'à la ligne de statut portant l'étiquette
    pub async fn read_response(&mut self, tag: &str) -> Response {
        let mut untagged = Vec::new();
        loop {
            let line = self.read_line().await.expect("connection closed");
            if line.starts_with(&format!("{tag} ")) {
                return Response {
                    untagged,
                    status: line,
                };
            }
            untagged.push(line);
        }
    }

    /// Lit des lignes jusqu'à en trouver une qui contient le texte
    pub async fn read_until_contains(&mut self, needle: &str) -> String {
        loop {
            let line = self.read_line().await.expect("connection closed");
            if line.contains(needle) {
                return line;
            }
        }
    }

    /// Envoie un UID SEARCH et renvoie la ligne de résultat
    pub async fn search_line(&mut self, criteria: &str) -> String {
        let response = self.command(&format!("UID SEARCH {criteria}")).await;
        response.line("SEARCH")
    }
}
