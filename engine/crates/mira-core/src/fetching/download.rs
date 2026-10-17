//! Téléchargement de la source d'un message depuis le serveur, sans le marquer comme lu.

use mira_vocabulary::error::codes::not_found;
use mira_vocabulary::{ErrorCategory, MiraError};

use super::access::ImapAccess;
use super::expected::require_expected_message;
use super::locate::SourceLocation;

/// Erreur : la position mémorisée ne désigne plus le message attendu
pub(super) fn stale_location(text: &str) -> MiraError {
    MiraError::new(ErrorCategory::NotFound, not_found::MESSAGE, text)
}

/// Ouvre une session, sélectionne la boîte et rend la source brute, après avoir vérifié qu'il s'agit bien du message
pub async fn download_source(
    access: &ImapAccess,
    location: &SourceLocation,
) -> Result<Vec<u8>, MiraError> {
    let mut client = access.connect(&location.account).await?;
    let selected = client.select(&location.mailbox_name, None).await?;
    if location
        .validity
        .is_some_and(|known| known != selected.uid_validity)
    {
        return Err(stale_location(
            "la boîte a changé de validité d'UID : resynchronisez-la avant de télécharger",
        ));
    }
    let source = client.fetch_source(location.uid).await?;
    client.logout().await?;
    let raw = source.ok_or_else(|| stale_location("le message n'existe plus sur le serveur"))?;
    require_expected_message(location, &raw)?;
    Ok(raw)
}
