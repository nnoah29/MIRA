//! Neutralisation des balises que le texte d'un mail ne doit pas pouvoir écrire : cadre du contexte, cadre d'un résultat d'outil.

/// Remplace le `<` de toute balise de ce nom (ouvrante ou fermante, casse et blancs quelconques) par `&lt;`.
pub(super) fn neutralize_tag(text: &str, tag: &str) -> String {
    let mut neutral = String::with_capacity(text.len());
    for (index, character) in text.char_indices() {
        let rest = text.get(index + 1..).unwrap_or_default();
        match character == '<' && is_tag(rest, tag) {
            true => neutral.push_str("&lt;"),
            false => neutral.push(character),
        }
    }
    neutral
}

/// Dit si le texte qui suit un `<` nomme cette balise, avec ou sans `/`, blancs tolérés.
fn is_tag(after_bracket: &str, tag: &str) -> bool {
    let rest = after_bracket.trim_start();
    let rest = rest.strip_prefix('/').unwrap_or(rest).trim_start();
    rest.get(..tag.len())
        .is_some_and(|word| word.eq_ignore_ascii_case(tag))
}
