//! Lecture simple des adresses d'en-tête pour ENVELOPE.

use crate::imap::response::quoting::nstring;

/// Découpe une liste d'adresses sur les virgules hors guillemets et chevrons
fn split_addresses(value: &str) -> Vec<String> {
    let (mut parts, mut current, mut quoted, mut angle) = (Vec::new(), String::new(), false, false);
    for c in value.chars() {
        match c {
            '"' => quoted = !quoted,
            '<' if !quoted => angle = true,
            '>' if !quoted => angle = false,
            ',' if !quoted && !angle => {
                parts.push(std::mem::take(&mut current));
                continue;
            }
            _ => {}
        }
        current.push(c);
    }
    parts.push(current);
    parts
}

/// Écrit une adresse `(nom NIL boîte hôte)`
fn format_address(text: &str) -> Option<String> {
    let text = text.trim();
    let (name, address) = match (text.find('<'), text.rfind('>')) {
        (Some(open), Some(close)) if open < close => {
            let name = text[..open].trim().trim_matches('"').trim();
            (Some(name).filter(|n| !n.is_empty()), &text[open + 1..close])
        }
        _ => (None, text),
    };
    let (mailbox, host) = address.trim().split_once('@')?;
    Some(format!(
        "({} NIL {} {})",
        nstring(name),
        nstring(Some(mailbox)),
        nstring(Some(host))
    ))
}

/// Liste d'adresses ENVELOPE, ou NIL
pub(crate) fn address_list(value: Option<&str>) -> String {
    let formatted: Vec<String> = split_addresses(value.unwrap_or(""))
        .iter()
        .filter_map(|part| format_address(part))
        .collect();
    if formatted.is_empty() {
        "NIL".to_string()
    } else {
        format!("({})", formatted.join(""))
    }
}
