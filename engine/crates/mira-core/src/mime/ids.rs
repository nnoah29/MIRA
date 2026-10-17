//! Identifiants de message (`Message-ID`, `In-Reply-To`, `References`) sans chevrons.

/// Retire espaces et chevrons autour d'un identifiant ; `None` s'il est vide.
pub(crate) fn clean_message_id(raw: &str) -> Option<String> {
    let trimmed = raw
        .trim()
        .trim_start_matches('<')
        .trim_end_matches('>')
        .trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}

/// Découpe un texte contenant plusieurs identifiants séparés par des espaces.
pub(crate) fn split_ids(text: &str) -> Vec<String> {
    text.split_whitespace()
        .filter_map(clean_message_id)
        .collect()
}
