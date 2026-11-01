//! Destinataires d'une proposition de brouillon : adresses validées et normalisées, vingt au plus par liste.

use mira_vocabulary::{AddressEntry, EmailAddress};

use super::render::cut;

pub(super) const MAX_RECIPIENTS: usize = 20;

/// Destinataires d'une liste ; une adresse invalide ou une liste trop longue est expliquée au modèle.
pub(super) fn recipients(
    list: Option<Vec<String>>,
    field: &str,
) -> Result<Option<Vec<AddressEntry>>, String> {
    let Some(list) = list else { return Ok(None) };
    if list.len() > MAX_RECIPIENTS {
        return Err(format!(
            "trop de destinataires dans « {field} » : {MAX_RECIPIENTS} au plus"
        ));
    }
    let parsed: Result<Vec<AddressEntry>, String> = list.iter().map(|t| entry(t)).collect();
    parsed.map(Some)
}

/// Une personne écrite `nom <adresse>` ou `adresse`.
fn entry(text: &str) -> Result<AddressEntry, String> {
    let (name, raw) = split(text.trim());
    let address =
        EmailAddress::parse(raw).map_err(|_| format!("adresse invalide : {}", cut(text, 80)))?;
    Ok(AddressEntry {
        name,
        address: address.as_str().to_string(),
    })
}

/// Sépare le nom affiché de l'adresse.
fn split(text: &str) -> (Option<String>, &str) {
    let Some((name, rest)) = text.rsplit_once('<') else {
        return (None, text);
    };
    if !rest.ends_with('>') {
        return (None, text);
    }
    let name = name.trim().trim_matches('"').trim();
    let shown = (!name.is_empty()).then(|| name.to_string());
    (shown, rest.trim_end_matches('>').trim())
}
