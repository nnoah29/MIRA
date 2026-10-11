//! Normalisation des noms de boîtes.

/// Nom canonique : INBOX est insensible à la casse, les autres noms sont conservés
pub(crate) fn normalize_name(name: &str) -> String {
    let trimmed = name.trim_end_matches('/');
    if trimmed.eq_ignore_ascii_case("INBOX") {
        "INBOX".to_string()
    } else {
        trimmed.to_string()
    }
}
