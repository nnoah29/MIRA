//! Nettoyage des noms et types de pièces jointes reçus : jamais de chemin, de contrôle ni d'inversion bidi.

const MAX_NAME_BYTES: usize = 255;
const MAX_TYPE_BYTES: usize = 255;
const FALLBACK_TYPE: &str = "application/octet-stream";

/// Indique si ce caractère doit disparaître d'un nom (contrôle, marque ou isolat bidirectionnel)
fn is_hidden_control(character: char) -> bool {
    character.is_control()
        || matches!(
            character,
            '\u{061C}' | '\u{200E}' | '\u{200F}' | '\u{202A}'..='\u{202E}' | '\u{2066}'..='\u{2069}'
        )
}

/// Coupe un texte à `max` octets sans scinder un caractère
fn truncate_on_boundary(text: &str, max: usize) -> &str {
    let mut end = text.len().min(max);
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    text.get(..end).unwrap_or("")
}

/// Nettoie un nom de fichier reçu : dernier composant, sans contrôle, 255 octets, jamais `.` ni `..`
pub(super) fn clean_filename(raw: &str) -> Option<String> {
    let last = raw.rsplit(['/', '\\', '\0']).next().unwrap_or("");
    let visible: String = last.chars().filter(|c| !is_hidden_control(*c)).collect();
    let trimmed = visible.trim();
    let bounded = truncate_on_boundary(trimmed, MAX_NAME_BYTES).trim_end();
    match bounded {
        "" | "." | ".." => None,
        name => Some(name.to_string()),
    }
}

/// Indique si ce caractère est admis dans un type MIME (jeton RFC 6838 et séparateur)
fn is_mime_character(character: char) -> bool {
    character.is_ascii_alphanumeric()
        || matches!(
            character,
            '/' | '.' | '+' | '-' | '_' | '!' | '#' | '$' | '&' | '^'
        )
}

/// Rend le type MIME s'il est bien formé `type/sous-type`, sinon `application/octet-stream`
pub(super) fn clean_mime_type(raw: String) -> String {
    let well_formed = raw.len() <= MAX_TYPE_BYTES
        && raw.chars().all(is_mime_character)
        && raw.split_once('/').is_some_and(|(kind, subtype)| {
            !kind.is_empty() && !subtype.is_empty() && !subtype.contains('/')
        });
    if well_formed {
        raw
    } else {
        FALLBACK_TYPE.to_string()
    }
}
