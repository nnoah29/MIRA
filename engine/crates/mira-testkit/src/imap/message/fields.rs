//! Champs d'en-tête d'un message brut.

use super::split::header_part;

/// Champ d'en-tête avec ses lignes de continuation.
#[derive(Debug, Clone)]
pub(crate) struct HeaderField {
    pub name: String,
    pub raw: Vec<u8>,
}

/// Nom du champ d'une ligne d'en-tête
fn field_name(line: &[u8]) -> String {
    let end = line
        .iter()
        .position(|byte| *byte == b':')
        .unwrap_or(line.len());
    String::from_utf8_lossy(&line[..end]).trim().to_string()
}

/// Découpe l'en-tête en champs en regroupant les lignes repliées
pub(crate) fn header_fields(header: &[u8]) -> Vec<HeaderField> {
    let mut fields: Vec<HeaderField> = Vec::new();
    for line in header.split_inclusive(|byte| *byte == b'\n') {
        if line == b"\r\n" || line == b"\n" {
            break;
        }
        let folded = matches!(line.first(), Some(b' ' | b'\t'));
        match (folded, fields.last_mut()) {
            (true, Some(last)) => last.raw.extend_from_slice(line),
            _ => fields.push(HeaderField {
                name: field_name(line),
                raw: line.to_vec(),
            }),
        }
    }
    fields
}

/// Champs choisis (ou tous sauf eux), suivis de la ligne vide
pub(crate) fn select_fields(header: &[u8], names: &[String], exclude: bool) -> Vec<u8> {
    let mut out = Vec::new();
    for field in header_fields(header) {
        let listed = names
            .iter()
            .any(|name| name.eq_ignore_ascii_case(&field.name));
        if listed != exclude {
            out.extend_from_slice(&field.raw);
        }
    }
    out.extend_from_slice(b"\r\n");
    out
}

/// Valeur dépliée d'un champ, sans tenir compte de la casse du nom
pub(crate) fn header_value(raw: &[u8], name: &str) -> Option<String> {
    let field = header_fields(header_part(raw))
        .into_iter()
        .find(|field| field.name.eq_ignore_ascii_case(name))?;
    let text = String::from_utf8_lossy(&field.raw).into_owned();
    let (_, value) = text.split_once(':')?;
    Some(
        value
            .lines()
            .map(str::trim)
            .collect::<Vec<_>>()
            .join(" ")
            .trim()
            .to_string(),
    )
}
