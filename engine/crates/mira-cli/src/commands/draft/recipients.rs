//! Lecture des destinataires saisis : `adresse` ou `Nom <adresse>`.

use mira_assembly::mira_vocabulary::{AddressEntry, MiraError};

use crate::argument_error::invalid_argument;

/// Lit une entrée `adresse` ou `Nom <adresse>`.
fn parse_entry(text: &str) -> Result<AddressEntry, MiraError> {
    let trimmed = text.trim();
    let (name, address) = match trimmed.split_once('<') {
        Some((name, rest)) => (
            Some(name.trim().trim_matches('"')),
            rest.trim_end_matches('>'),
        ),
        None => (None, trimmed),
    };
    if !address.contains('@') {
        return Err(invalid_argument("destinataire invalide : adresse attendue"));
    }
    Ok(AddressEntry {
        name: name.filter(|label| !label.is_empty()).map(str::to_string),
        address: address.trim().to_string(),
    })
}

/// Liste de destinataires : absente sans option, vide avec une valeur vide (efface la liste).
pub fn recipients(values: &[String]) -> Result<Option<Vec<AddressEntry>>, MiraError> {
    if values.is_empty() {
        return Ok(None);
    }
    values
        .iter()
        .filter(|value| !value.trim().is_empty())
        .map(|value| parse_entry(value))
        .collect::<Result<Vec<_>, _>>()
        .map(Some)
}
