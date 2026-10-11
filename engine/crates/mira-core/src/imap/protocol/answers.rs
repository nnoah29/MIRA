//! Commande qui réclame des réponses du client (authentification SASL).

use mira_vocabulary::MiraError;

use super::response::Response;
use super::wire::Wire;
use crate::imap::parse::status::tagged_status;

impl Wire {
    /// Envoie une commande puis répond à chaque invite `+` avec les réponses données, puis par une ligne vide.
    pub(crate) async fn command_with_answers(
        &mut self,
        line: &str,
        answers: &[String],
    ) -> Result<Response, MiraError> {
        let tag = self.fresh_tag();
        self.write_bytes(format!("{tag} {line}\r\n").as_bytes())
            .await?;
        let mut remaining = answers.iter();
        let mut untagged = Vec::new();
        loop {
            let reply = self.read_line().await?;
            if let Some(status) = tagged_status(&reply, &tag) {
                return Ok(Response { untagged, status });
            }
            if reply.starts_with(b"+") {
                let answer = remaining.next().map_or("", String::as_str);
                self.write_bytes(format!("{answer}\r\n").as_bytes()).await?;
            } else if let Some(rest) = reply.strip_prefix(b"* ") {
                untagged.push(rest.to_vec());
            }
        }
    }
}
