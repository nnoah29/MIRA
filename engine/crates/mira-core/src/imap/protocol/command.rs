//! Envoi d'une commande étiquetée et lecture de sa réponse.

use mira_vocabulary::MiraError;

use super::response::Response;
use super::wire::Wire;
use crate::imap::parse::status::tagged_status;

impl Wire {
    /// Lit les réponses jusqu'à la ligne de statut portant l'étiquette.
    pub(crate) async fn read_until_tag(&mut self, tag: &str) -> Result<Response, MiraError> {
        let mut untagged = Vec::new();
        loop {
            let line = self.read_line().await?;
            if let Some(status) = tagged_status(&line, tag) {
                return Ok(Response { untagged, status });
            }
            if let Some(rest) = line.strip_prefix(b"* ") {
                untagged.push(rest.to_vec());
            }
        }
    }

    /// Envoie une commande sur une ligne et rend sa réponse complète.
    pub(crate) async fn command(&mut self, line: &str) -> Result<Response, MiraError> {
        let tag = self.fresh_tag();
        self.write_bytes(format!("{tag} {line}\r\n").as_bytes())
            .await?;
        self.read_until_tag(&tag).await
    }
}
