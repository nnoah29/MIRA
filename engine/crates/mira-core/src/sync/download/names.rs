//! Lecture des réponses LIST : boîtes sélectionnables, nom court et nom du parent.

use mira_vocabulary::RawMailbox;

/// Une boîte « non sélectionnable » n'a pas de messages : elle n'entre pas dans le miroir.
pub fn is_selectable(raw: &RawMailbox) -> bool {
    let blocked = |attr: &String| {
        attr.eq_ignore_ascii_case("\\Noselect") || attr.eq_ignore_ascii_case("\\NonExistent")
    };
    !raw.attributes.iter().any(blocked)
}

/// Nom court d'une boîte (dernier segment) et nom du parent d'après le séparateur.
pub fn split_name(raw: &RawMailbox) -> (String, Option<String>) {
    let split = raw.delimiter.as_deref().filter(|d| !d.is_empty());
    match split.and_then(|delimiter| raw.name.rsplit_once(delimiter)) {
        Some((parent, short)) => (short.to_string(), Some(parent.to_string())),
        None => (raw.name.clone(), None),
    }
}
