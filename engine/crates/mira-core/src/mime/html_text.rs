//! Texte brut tiré d'un document HTML : balises retirées, entités décodées.

use super::html_entities::decode_entities;

const BLOCK_TAGS: [&str; 9] = ["br", "p", "div", "li", "tr", "h1", "h2", "h3", "blockquote"];
const HIDDEN_TAGS: [&str; 2] = ["script", "style"];
const MAX_HTML_BYTES: usize = 2 * 1024 * 1024;

/// Nom en minuscules d'une balise (sans `/` initial ni attributs).
fn tag_name(tag: &str) -> String {
    tag.trim_start_matches('/')
        .split(|c: char| c.is_whitespace() || c == '/')
        .next()
        .unwrap_or_default()
        .to_ascii_lowercase()
}

/// Position de la prochaine balise fermante du nom donné, sans tenir compte de la casse.
fn closing_tag_start(html: &str, name: &str) -> Option<usize> {
    let mut from = 0;
    while let Some(offset) = html[from..].find("</") {
        let at = from + offset;
        let candidate = html.as_bytes().get(at + 2..at + 2 + name.len());
        if candidate.is_some_and(|bytes| bytes.eq_ignore_ascii_case(name.as_bytes())) {
            return Some(at);
        }
        from = at + 2;
    }
    None
}

/// Longueur à sauter pour passer l'élément masqué jusqu'à sa balise fermante.
fn hidden_length(html: &str, name: &str) -> usize {
    closing_tag_start(html, name)
        .and_then(|at| html[at..].find('>').map(|end| at + end + 1))
        .unwrap_or(html.len())
}

/// Retire les balises en gardant un saut de ligne pour les balises de bloc.
fn strip_tags(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut rest = html;
    while let Some(start) = rest.find('<') {
        out.push_str(&rest[..start]);
        let Some(end) = rest[start..].find('>') else {
            return out;
        };
        let name = tag_name(&rest[start + 1..start + end]);
        let after = start + end + 1;
        let opening = !rest[start + 1..].starts_with('/');
        let skip = if opening && HIDDEN_TAGS.contains(&name.as_str()) {
            hidden_length(&rest[after..], &name)
        } else {
            0
        };
        rest = &rest[after + skip..];
        if BLOCK_TAGS.contains(&name.as_str()) {
            out.push('\n');
        }
    }
    out.push_str(rest);
    out
}

/// Début du document, au plus `MAX_HTML_BYTES` octets, coupé sur un caractère entier.
fn capped(html: &str) -> &str {
    &html[..html.floor_char_boundary(MAX_HTML_BYTES)]
}

/// Convertit un document HTML en texte lisible quand il n'existe pas de partie texte.
pub fn html_to_text(html: &str) -> String {
    let stripped = decode_entities(&strip_tags(capped(html)));
    let lines: Vec<&str> = stripped.lines().map(str::trim).collect();
    let mut text = lines.join("\n");
    while text.contains("\n\n\n") {
        text = text.replace("\n\n\n", "\n\n");
    }
    text.trim().to_string()
}
