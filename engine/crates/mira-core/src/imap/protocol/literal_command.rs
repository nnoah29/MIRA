//! Commande dont le dernier argument est un littéral (APPEND).

use mira_vocabulary::MiraError;

use super::response::Response;
use super::wire::Wire;
use crate::imap::parse::status::tagged_status;

impl Wire {
    /// Attend l'invite `+` du serveur ; rend la réponse finale si le serveur refuse avant.
    async fn await_go_ahead(&mut self, tag: &str) -> Result<Option<Response>, MiraError> {
        let mut untagged = Vec::new();
        loop {
            let reply = self.read_line().await?;
            if let Some(status) = tagged_status(&reply, tag) {
                return Ok(Some(Response { untagged, status }));
            }
            if reply.starts_with(b"+") {
                return Ok(None);
            }
            if let Some(rest) = reply.strip_prefix(b"* ") {
                untagged.push(rest.to_vec());
            }
        }
    }

    /// Envoie `préfixe {n}` puis les octets du littéral, avec ou sans attente de l'invite.
    pub(crate) async fn command_with_literal(
        &mut self,
        prefix: &str,
        data: &[u8],
    ) -> Result<Response, MiraError> {
        let tag = self.fresh_tag();
        let plus = if self.literal_plus { "+" } else { "" };
        let header = format!("{tag} {prefix} {{{}{plus}}}\r\n", data.len());
        self.write_bytes(header.as_bytes()).await?;
        if !self.literal_plus
            && let Some(refused) = self.await_go_ahead(&tag).await?
        {
            return Ok(refused);
        }
        let mut payload = data.to_vec();
        payload.extend_from_slice(b"\r\n");
        self.write_bytes(&payload).await?;
        self.read_until_tag(&tag).await
    }
}
