//! Téléchargement de la source d'un message depuis le serveur, sans le marquer comme lu.

use mira_vocabulary::error::codes::not_found;
use mira_vocabulary::{ErrorCategory, MiraError};

use super::access::ImapAccess;
use super::locate::SourceLocation;

/// Ouvre une session, sélectionne la boîte et rend la source brute du message
pub async fn download_source(
    access: &ImapAccess,
    location: &SourceLocation,
) -> Result<Vec<u8>, MiraError> {
    let mut client = access.connect(&location.account).await?;
    client.select(&location.mailbox_name, None).await?;
    let source = client.fetch_source(location.uid).await?;
    client.logout().await?;
    source.ok_or_else(|| {
        MiraError::new(
            ErrorCategory::NotFound,
            not_found::MESSAGE,
            "le message n'existe plus sur le serveur",
        )
    })
}
