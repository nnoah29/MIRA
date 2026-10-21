//! Bornes d'entrée de l'assainissement : taille et profondeur d'imbrication (AG-34).

const MAX_HTML_BYTES: usize = 2 * 1024 * 1024;
const MAX_NESTING_DEPTH: usize = 512;
const VOID_TAGS: [&str; 14] = [
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param", "source",
    "track", "wbr",
];

/// Dit si le HTML reste dans les bornes de taille et de profondeur
pub(super) fn is_within_limits(input: &str) -> bool {
    input.len() <= MAX_HTML_BYTES && !exceeds_nesting_depth(input)
}

/// Dit si le nombre de balises ouvertes en même temps dépasse la borne
fn exceeds_nesting_depth(input: &str) -> bool {
    let mut depth = 0usize;
    for tag in input.split('<').skip(1) {
        if tag.starts_with('/') {
            depth = depth.saturating_sub(1);
        } else if opens_element(tag) {
            depth += 1;
        }
        if depth > MAX_NESTING_DEPTH {
            return true;
        }
    }
    false
}

/// Dit si le texte suivant un `<` ouvre un élément qui contiendra des enfants
fn opens_element(after_bracket: &str) -> bool {
    let name: String = after_bracket
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric())
        .collect::<String>()
        .to_ascii_lowercase();
    !name.is_empty() && !VOID_TAGS.contains(&name.as_str())
}

/// Rend le texte du message, tronqué et échappé, sans aucune balise active
pub(super) fn escaped_text_fallback(input: &str) -> String {
    let mut end = input.len().min(MAX_HTML_BYTES);
    while !input.is_char_boundary(end) {
        end -= 1;
    }
    format!("<pre>{}</pre>", ammonia::clean_text(&input[..end]))
}
