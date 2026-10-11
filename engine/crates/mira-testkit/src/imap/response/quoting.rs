//! Chaînes entre guillemets et atomes.

/// Chaîne entre guillemets avec échappements
pub(crate) fn quote(text: &str) -> String {
    format!("\"{}\"", text.replace('\\', "\\\\").replace('"', "\\\""))
}

/// Atome si possible, sinon chaîne entre guillemets
pub(crate) fn astring(text: &str) -> String {
    let plain = |c: char| c.is_ascii_alphanumeric() || "\\-_.$/@:+".contains(c);
    if !text.is_empty() && text.chars().all(plain) {
        text.to_string()
    } else {
        quote(text)
    }
}

/// Chaîne entre guillemets ou NIL
pub(crate) fn nstring(text: Option<&str>) -> String {
    text.map_or_else(|| "NIL".to_string(), quote)
}
