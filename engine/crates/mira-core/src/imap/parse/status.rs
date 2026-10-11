//! Lignes de statut (`OK`, `NO`, `BAD`, `PREAUTH`, `BYE`) et codes de réponse entre crochets.

use crate::imap::protocol::response::{Status, StatusKind};

/// Code de réponse entre crochets, par exemple `[UIDVALIDITY 5]`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ResponseCode {
    pub name: String,
    pub args: String,
}

/// Découpe le contenu d'un code de réponse en nom (majuscules) et arguments.
pub(crate) fn parse_code(inner: &str) -> ResponseCode {
    let trimmed = inner.trim();
    let (name, args) = trimmed.split_once(' ').unwrap_or((trimmed, ""));
    ResponseCode {
        name: name.to_ascii_uppercase(),
        args: args.trim().to_string(),
    }
}

/// Sépare le code de réponse éventuel du texte libre qui suit le mot de statut.
pub(crate) fn parse_status_text(rest: &str) -> (Option<ResponseCode>, String) {
    let rest = rest.trim_start();
    let coded = rest
        .strip_prefix('[')
        .and_then(|after| after.split_once(']'));
    match coded {
        Some((inner, text)) => (Some(parse_code(inner)), text.trim().to_string()),
        None => (None, rest.trim().to_string()),
    }
}

/// Lit une ligne non étiquetée de statut : mot, code éventuel, texte.
pub(crate) fn untagged_status(line: &[u8]) -> Option<(String, Option<ResponseCode>, String)> {
    let text = String::from_utf8_lossy(line);
    let (word, rest) = text.split_once(' ').unwrap_or((&text, ""));
    let word = word.to_ascii_uppercase();
    if !matches!(word.as_str(), "OK" | "NO" | "BAD" | "PREAUTH" | "BYE") {
        return None;
    }
    let (code, text) = parse_status_text(rest);
    Some((word, code, text))
}

/// Lit la ligne de statut qui porte l'étiquette donnée.
pub(crate) fn tagged_status(line: &[u8], tag: &str) -> Option<Status> {
    let text = String::from_utf8_lossy(line);
    let rest = text.strip_prefix(tag)?.strip_prefix(' ')?;
    let (word, tail) = rest.split_once(' ').unwrap_or((rest, ""));
    let kind = match word.to_ascii_uppercase().as_str() {
        "OK" => StatusKind::Ok,
        "NO" => StatusKind::No,
        "BAD" => StatusKind::Bad,
        _ => return None,
    };
    let (code, text) = parse_status_text(tail);
    Some(Status { kind, code, text })
}
