//! Vérifie qu'une source téléchargée est bien celle du message demandé (un UID périmé en désignerait un autre).

use mira_vocabulary::MiraError;

use super::download::stale_location;
use super::locate::SourceLocation;
use crate::mime::parse_headers;

const MAX_HEADER_BYTES: usize = 256 * 1024;

/// Identifiant de message sans chevrons ni espaces
fn bare(id: &str) -> &str {
    id.trim().trim_start_matches('<').trim_end_matches('>')
}

/// Octets d'en-tête de la source : jusqu'à la première ligne vide, bornés
fn header_part(raw: &[u8]) -> &[u8] {
    let limit = raw.len().min(MAX_HEADER_BYTES);
    let head = &raw[..limit];
    match head.windows(4).position(|window| window == b"\r\n\r\n") {
        Some(end) => &head[..end + 4],
        None => head,
    }
}

/// Refuse la source si son `Message-ID` diffère de celui que le miroir connaît pour ce message
pub(super) fn require_expected_message(
    location: &SourceLocation,
    raw: &[u8],
) -> Result<(), MiraError> {
    let Some(expected) = &location.message.message_id_header else {
        return Ok(());
    };
    let found = parse_headers(header_part(raw)).message_id;
    if found.as_deref().map(bare) == Some(bare(expected)) {
        return Ok(());
    }
    Err(stale_location(
        "le message trouvé sous cet UID n'est pas celui demandé : resynchronisez la boîte",
    ))
}
