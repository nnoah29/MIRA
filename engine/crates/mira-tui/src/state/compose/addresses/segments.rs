//! Découpe d'un champ d'adresses en segments séparés par `,` ou `;` hors guillemets et crochets.

/// Séparateurs d'adresses reconnus hors guillemets et crochets.
fn is_separator(c: char) -> bool {
    c == ',' || c == ';'
}

/// Segments non vides du texte, nettoyés de leurs espaces de bordure.
pub fn split_segments(text: &str) -> Vec<String> {
    let mut segments = Vec::new();
    let mut current = String::new();
    let mut quoted = false;
    let mut escaped = false;
    let mut angle = false;
    for c in text.chars() {
        let splits = is_separator(c) && !quoted && !angle;
        if splits {
            push_segment(&mut segments, &mut current);
            continue;
        }
        track_context(c, &mut quoted, &mut escaped, &mut angle);
        current.push(c);
    }
    push_segment(&mut segments, &mut current);
    segments
}

/// Met à jour l'état « dans des guillemets / après un antislash / dans des crochets » après ce caractère.
fn track_context(c: char, quoted: &mut bool, escaped: &mut bool, angle: &mut bool) {
    if *escaped {
        *escaped = false;
    } else if *quoted && c == '\\' {
        *escaped = true;
    } else if c == '"' {
        *quoted = !*quoted;
    } else if !*quoted && c == '<' {
        *angle = true;
    } else if !*quoted && c == '>' {
        *angle = false;
    }
}

/// Range le segment courant s'il n'est pas vide, puis le vide.
fn push_segment(segments: &mut Vec<String>, current: &mut String) {
    let trimmed = current.trim();
    if !trimmed.is_empty() {
        segments.push(trimmed.to_string());
    }
    current.clear();
}
