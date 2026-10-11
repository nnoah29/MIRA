//! Interrogation des capacités du serveur.

use mira_vocabulary::MiraError;

use crate::imap::errors::status::Operation;
use crate::imap::parse::capability::capabilities_of;
use crate::imap::protocol::wire::Wire;

/// Demande la liste des capacités au serveur.
pub(crate) async fn fetch_capabilities(wire: &mut Wire) -> Result<Vec<String>, MiraError> {
    let response = wire
        .command("CAPABILITY")
        .await?
        .require_ok(Operation::Plain)?;
    Ok(capabilities_of(&response.untagged).unwrap_or_default())
}
