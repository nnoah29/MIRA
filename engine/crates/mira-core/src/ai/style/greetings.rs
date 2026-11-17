//! Ouvertures des mails (« Bonjour », « Salut »…) : comptées sans le nom qui suit, jamais de donnée personnelle.

const OPENERS: [&str; 10] = [
    "bonjour", "bonsoir", "salut", "coucou", "hello", "hi", "hey", "cher", "chère", "dear",
];

/// Majuscule sur la première lettre.
fn capitalized(word: &str) -> String {
    let mut chars = word.chars();
    chars
        .next()
        .map(|first| first.to_uppercase().chain(chars).collect())
        .unwrap_or_default()
}

/// Mot d'ouverture de ce mail (première ligne, premier mot), s'il est connu.
pub fn greeting_of(cleaned: &str) -> Option<String> {
    let first_line = cleaned.lines().next()?;
    let word = first_line
        .split(|c: char| !c.is_alphabetic())
        .find(|w| !w.is_empty())?;
    let lower = word.to_lowercase();
    OPENERS
        .contains(&lower.as_str())
        .then(|| capitalized(&lower))
}

/// Les `limit` ouvertures les plus fréquentes, à égalité dans l'ordre de première apparition.
pub fn top_greetings(found: &[String], limit: usize) -> Vec<String> {
    let mut counted: Vec<(String, usize)> = Vec::new();
    for greeting in found {
        match counted.iter_mut().find(|(g, _)| g == greeting) {
            Some(entry) => entry.1 += 1,
            None => counted.push((greeting.clone(), 1)),
        }
    }
    counted.sort_by_key(|entry| std::cmp::Reverse(entry.1));
    counted.into_iter().take(limit).map(|(g, _)| g).collect()
}
