//! Dépôt d'un message dans une boîte (APPEND), avec l'UID attribué si le serveur l'annonce.

use mira_vocabulary::MiraError;

use crate::imap::client::ImapClient;
use crate::imap::errors::status::Operation;
use crate::imap::protocol::quoting::quote;
use crate::imap::protocol::response::Response;
use crate::imap::types::ImapFlag;

/// Début de la commande APPEND : boîte et drapeaux éventuels.
fn append_prefix(mailbox: &str, flags: &[ImapFlag]) -> Result<String, MiraError> {
    let name = quote(mailbox)?;
    if flags.is_empty() {
        return Ok(format!("APPEND {name}"));
    }
    let names: Vec<&str> = flags.iter().map(ImapFlag::as_str).collect();
    Ok(format!("APPEND {name} ({})", names.join(" ")))
}

/// UID attribué, lu dans le code `[APPENDUID validité uid]`.
fn appended_uid(response: &Response) -> Option<u32> {
    let code = response.status.code.as_ref()?;
    if code.name != "APPENDUID" {
        return None;
    }
    let uid_set = code.args.split_whitespace().nth(1)?;
    uid_set.split([':', ',']).next()?.parse().ok()
}

impl ImapClient {
    /// Dépose le message dans la boîte ; rend son UID si le serveur le communique (UIDPLUS).
    pub async fn append(
        &mut self,
        mailbox: &str,
        flags: &[ImapFlag],
        raw: &[u8],
    ) -> Result<Option<u32>, MiraError> {
        let prefix = append_prefix(mailbox, flags)?;
        let response = self
            .wire
            .command_with_literal(&prefix, raw)
            .await?
            .require_ok(Operation::Plain)?;
        Ok(appended_uid(&response))
    }
}
