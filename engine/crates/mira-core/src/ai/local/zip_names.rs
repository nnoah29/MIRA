//! Noms d'entrées d'une archive `.zip` : seuls des chemins relatifs sûrs sous Windows comme sous Linux.

use std::path::Path;

use mira_vocabulary::MiraError;

use super::extract_check::{check_relative, corrupted};

const RESERVED: [&str; 4] = ["CON", "PRN", "AUX", "NUL"];

/// Refuse un nom vide, absolu, avec antislash, `:` ou `..`, ou un composant réservé à Windows.
pub(super) fn check_zip_name(name: &str) -> Result<(), MiraError> {
    if name.is_empty() || name.contains(['\\', ':']) {
        return Err(corrupted());
    }
    check_relative(Path::new(name))?;
    if name.split('/').any(is_reserved_component) {
        return Err(corrupted());
    }
    Ok(())
}

/// Un composant est réservé s'il porte un nom de périphérique Windows ou finit par un point ou une espace.
fn is_reserved_component(component: &str) -> bool {
    if component.ends_with('.') && component != "." || component.ends_with(' ') {
        return true;
    }
    let stem = component
        .split('.')
        .next()
        .unwrap_or("")
        .to_ascii_uppercase();
    let numbered = ["COM", "LPT"].iter().any(|p| {
        stem.strip_prefix(p)
            .is_some_and(|n| n.len() == 1 && n != "0" && n.bytes().all(|b| b.is_ascii_digit()))
    });
    RESERVED.contains(&stem.as_str()) || numbered
}
