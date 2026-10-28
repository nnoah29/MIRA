//! Mention explicite : mon adresse ou `@` suivi de mon nom, dans le texte non cité.

use super::me::Me;

/// Le texte, hors lignes citées, contient-il mon adresse ou `@` suivi de l'un de mes noms ?
pub fn mentions_me(text: &str, me: &Me) -> bool {
    let body = unquoted(text).to_lowercase();
    let by_address = me.addresses.iter().any(|a| appears(&body, a));
    let by_name = me.names.iter().any(|n| appears(&body, &format!("@{n}")));
    by_address || by_name
}

/// Texte sans les lignes de citation (celles qui commencent par `>`) ni la signature (après `-- `)
fn unquoted(text: &str) -> String {
    let kept: Vec<&str> = text
        .lines()
        .take_while(|line| line.trim_end() != "--")
        .filter(|line| !line.trim_start().starts_with('>'))
        .collect();
    kept.join("\n")
}

/// `needle` apparaît-il entier, sans caractère de mot collé avant ni après ?
fn appears(body: &str, needle: &str) -> bool {
    body.match_indices(needle).any(|(start, found)| {
        let before = body[..start].chars().next_back();
        let after = &body[start + found.len()..];
        !before.is_some_and(is_before_char) && !continues(after)
    })
}

/// Caractère qui, juste avant une adresse ou un `@nom`, en fait une partie d'un autre mot
fn is_before_char(c: char) -> bool {
    c.is_alphanumeric() || matches!(c, '.' | '_' | '-' | '@' | '+' | '%')
}

/// Le texte qui suit prolonge-t-il le mot (lettre, tiret, ou point suivi d'une lettre) ?
fn continues(after: &str) -> bool {
    let mut chars = after.chars();
    match chars.next() {
        Some('.') => chars.next().is_some_and(char::is_alphanumeric),
        Some(c) => c.is_alphanumeric() || matches!(c, '_' | '-'),
        None => false,
    }
}
