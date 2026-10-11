//! Passage au chiffrement par STARTTLS sur une connexion ouverte en clair.

use mira_vocabulary::MiraError;

use super::tls::wrap_tls;
use crate::imap::errors::status::Operation;
use crate::imap::protocol::wire::Wire;

/// Négocie STARTTLS puis rend un flux chiffré sur la même connexion.
pub(crate) async fn upgrade_to_tls(mut wire: Wire, host: &str) -> Result<Wire, MiraError> {
    let wait = wire.wait;
    wire.command("STARTTLS")
        .await?
        .require_ok(Operation::Extension)?;
    let secured = wrap_tls(wire.into_transport(), host).await?;
    Ok(Wire::new(secured, wait))
}
