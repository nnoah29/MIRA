//! Mesures d'un texte : mots, phrases, points d'exclamation, émojis, médiane.

/// Nombre de mots (séparés par des blancs).
pub fn word_count(text: &str) -> u32 {
    u32::try_from(text.split_whitespace().count()).unwrap_or(u32::MAX)
}

/// Mots de chaque phrase non vide ; une phrase finit à `.`, `!`, `?` ou à la fin de la ligne.
pub fn sentence_word_counts(text: &str) -> Vec<u32> {
    text.split(['.', '!', '?', '\n'])
        .map(word_count)
        .filter(|count| *count > 0)
        .collect()
}

/// Nombre de points d'exclamation.
pub fn exclamations(text: &str) -> u32 {
    u32::try_from(text.matches('!').count()).unwrap_or(u32::MAX)
}

/// Dit si le texte contient un émoji ou un pictogramme.
pub fn has_emoji(text: &str) -> bool {
    text.chars()
        .any(|c| matches!(c as u32, 0x1F300..=0x1FAFF | 0x2600..=0x27BF))
}

/// Médiane basse (0 sans valeur).
pub fn median(values: &mut [u32]) -> u32 {
    values.sort_unstable();
    values
        .get(values.len().saturating_sub(1) / 2)
        .copied()
        .unwrap_or(0)
}
