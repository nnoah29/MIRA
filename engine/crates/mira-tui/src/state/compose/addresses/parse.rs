//! Lecture d'un champ d'adresses : `a@b.c`, `<a@b.c>` ou `Nom <a@b.c>` séparés par des virgules.

use mira_assembly::mira_vocabulary::{AddressEntry, EmailAddress};

use super::segments::split_segments;

/// Adresses du texte ; en cas d'échec, le premier segment invalide tel qu'il a été saisi.
pub fn parse_addresses(text: &str) -> Result<Vec<AddressEntry>, String> {
    split_segments(text)
        .into_iter()
        .map(|segment| parse_segment(&segment).ok_or(segment))
        .collect()
}

/// Une adresse, avec son nom d'affichage s'il y en a un ; `None` si la forme est invalide.
fn parse_segment(segment: &str) -> Option<AddressEntry> {
    let (name, address) = match segment.find('<') {
        None => (None, segment),
        Some(open) => {
            let inner = segment.get(open + 1..)?.strip_suffix('>')?;
            (Some(read_name(segment.get(..open)?)?), inner)
        }
    };
    let address = address.trim();
    let forbidden = address.contains(['<', '>', '"', ',', ';']);
    if forbidden || EmailAddress::parse(address).is_err() {
        return None;
    }
    Some(AddressEntry {
        name: name.filter(|name| !name.is_empty()),
        address: address.to_string(),
    })
}

/// Nom d'affichage : texte nu, ou texte entre guillemets avec `\"` et `\\` déséchappés ; `None` si mal formé.
fn read_name(raw: &str) -> Option<String> {
    let raw = raw.trim();
    let Some(inner) = raw.strip_prefix('"') else {
        return (!raw.contains('"')).then(|| raw.to_string());
    };
    let inner = inner.strip_suffix('"')?;
    let mut name = String::new();
    let mut chars = inner.chars();
    while let Some(c) = chars.next() {
        match c {
            '\\' => name.push(chars.next()?),
            '"' => return None,
            c => name.push(c),
        }
    }
    Some(name)
}
