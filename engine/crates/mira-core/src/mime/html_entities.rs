//! Décodage des entités HTML courantes.

/// Remplace l'entité nommée ou numérique (sans `&` ni `;`) par son caractère.
fn entity_char(name: &str) -> Option<char> {
    match name {
        "amp" => Some('&'),
        "lt" => Some('<'),
        "gt" => Some('>'),
        "quot" => Some('"'),
        "apos" => Some('\''),
        "nbsp" => Some(' '),
        _ => numeric_entity(name),
    }
}

/// Entité numérique décimale (`#233`) ou hexadécimale (`#xE9`).
fn numeric_entity(name: &str) -> Option<char> {
    let number = name.strip_prefix('#')?;
    let code = match number.strip_prefix(['x', 'X']) {
        Some(hex) => u32::from_str_radix(hex, 16).ok()?,
        None => number.parse().ok()?,
    };
    char::from_u32(code)
}

/// Décode les entités d'un texte ; une entité inconnue est laissée telle quelle.
pub(crate) fn decode_entities(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(start) = rest.find('&') {
        out.push_str(&rest[..start]);
        let tail = &rest[start + 1..];
        let decoded = tail
            .find(';')
            .filter(|end| *end <= 8)
            .and_then(|end| entity_char(&tail[..end]).map(|ch| (ch, end)));
        match decoded {
            Some((ch, end)) => {
                out.push(ch);
                rest = &tail[end + 1..];
            }
            None => {
                out.push('&');
                rest = tail;
            }
        }
    }
    out.push_str(rest);
    out
}
