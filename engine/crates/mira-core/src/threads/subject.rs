//! Normalisation de l'objet d'un message : retrait des préfixes de réponse et de transfert.

const PREFIXES: [&str; 8] = ["re", "fwd", "fw", "tr", "aw", "wg", "réf", "ref"];

/// Retire un préfixe de réponse ou de transfert en tête de l'objet, s'il y en a un
fn strip_one_prefix(subject: &str) -> Option<&str> {
    let trimmed = subject.trim_start();
    PREFIXES
        .iter()
        .find_map(|prefix| strip_prefix_word(trimmed, prefix))
}

/// Retire `<mot> :` (casse indifférente, espaces avant les deux-points tolérés) en tête du texte
fn strip_prefix_word<'a>(text: &'a str, prefix: &str) -> Option<&'a str> {
    let count = prefix.chars().count();
    let head: String = text.chars().take(count).collect::<String>().to_lowercase();
    if head != prefix {
        return None;
    }
    let rest = text
        .char_indices()
        .nth(count)
        .map_or("", |(index, _)| text.get(index..).unwrap_or(""));
    rest.trim_start().strip_prefix(':')
}

/// Retire tous les préfixes de réponse et de transfert en tête de l'objet, en gardant la casse
pub fn strip_reply_prefixes(subject: &str) -> &str {
    let mut current = subject;
    while let Some(rest) = strip_one_prefix(current) {
        current = rest;
    }
    current.trim()
}

/// Objet normalisé pour comparer des messages : sans préfixes, en minuscules, espaces simplifiés
pub fn normalize_subject(subject: &str) -> String {
    strip_reply_prefixes(subject)
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}
