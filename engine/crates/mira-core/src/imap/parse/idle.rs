//! Annonces spontanées du serveur pendant IDLE.

/// Indique si la ligne annonce un changement de la boîte (arrivée, suppression, drapeaux).
pub(crate) fn is_mailbox_change(line: &[u8]) -> bool {
    let head = String::from_utf8_lossy(&line[..line.len().min(48)]).to_ascii_uppercase();
    let mut words = head.split_whitespace();
    match (words.next(), words.next()) {
        (Some("VANISHED"), _) => true,
        (Some(number), Some(word)) if number.bytes().all(|b| b.is_ascii_digit()) => {
            matches!(word, "EXISTS" | "EXPUNGE" | "FETCH")
        }
        _ => false,
    }
}
