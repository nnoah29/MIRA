//! Traduction des attributs FETCH `X-GM-*` en identifiant stable, fil et libellés.

use mira_vocabulary::{RawMessageAttributes, ServerMessageTranslation};

const MESSAGE_ID: &str = "X-GM-MSGID";
const THREAD_ID: &str = "X-GM-THRID";
const LABELS: &str = "X-GM-LABELS";

/// Lit un libellé entre guillemets (échappements `\\` et `\"` décodés) et rend le reste.
fn quoted_label(rest: &str) -> (String, &str) {
    let mut label = String::new();
    let mut chars = rest.char_indices().skip(1);
    while let Some((index, c)) = chars.next() {
        match c {
            '\\' => label.extend(chars.next().map(|(_, escaped)| escaped)),
            '"' => return (label, &rest[index + 1..]),
            other => label.push(other),
        }
    }
    (label, "")
}

/// Lit un libellé sans guillemets, jusqu'au prochain espace, et rend le reste.
fn atom_label(rest: &str) -> (String, &str) {
    let end = rest.find(char::is_whitespace).unwrap_or(rest.len());
    (rest[..end].to_string(), &rest[end..])
}

/// Découpe la liste parenthésée d'un `X-GM-LABELS` (libellés système comme `\Inbox` conservés).
pub fn parse_labels(value: &str) -> Vec<String> {
    let trimmed = value.trim();
    let inner = trimmed.strip_prefix('(').unwrap_or(trimmed);
    let mut rest = inner.strip_suffix(')').unwrap_or(inner).trim_start();
    let mut labels = Vec::new();
    while !rest.is_empty() {
        let (label, remaining) = if rest.starts_with('"') {
            quoted_label(rest)
        } else {
            atom_label(rest)
        };
        labels.push(label);
        rest = remaining.trim_start();
    }
    labels
}

/// Garde une valeur non vide, débarrassée de ses espaces de bordure.
fn non_empty(value: &str) -> Option<String> {
    let trimmed = value.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}

/// Traduit les attributs bruts du service ; les attributs inconnus sont ignorés.
pub fn translate(raw: &RawMessageAttributes) -> ServerMessageTranslation {
    let mut translation = ServerMessageTranslation::default();
    for (name, value) in &raw.items {
        match name.to_ascii_uppercase().as_str() {
            MESSAGE_ID => translation.stable_id = non_empty(value),
            THREAD_ID => translation.thread_hint = non_empty(value),
            LABELS => translation.labels = parse_labels(value),
            _ => {}
        }
    }
    translation
}
