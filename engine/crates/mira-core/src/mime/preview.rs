//! Aperçu d'un message : début du texte, espaces normalisés.

const PREVIEW_CHARS: usize = 200;

/// Rend environ 200 caractères du texte, espaces normalisés, coupés sur un caractère entier.
pub fn preview_text(text: &str) -> String {
    let normalized = text.split_whitespace().collect::<Vec<_>>().join(" ");
    normalized.chars().take(PREVIEW_CHARS).collect()
}
