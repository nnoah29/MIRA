//! Conversion des adresses de `mail-parser` vers le vocabulaire du moteur.

use mail_parser::{Addr, Address};
use mira_vocabulary::AddressEntry;

/// Convertit une adresse d'en-tête ; une adresse sans partie utilisable est ignorée.
fn entry_of(addr: &Addr<'_>) -> Option<AddressEntry> {
    let name = addr
        .name
        .as_deref()
        .map(str::trim)
        .filter(|text| !text.is_empty())
        .map(str::to_string);
    let address = addr
        .address
        .as_deref()
        .map(str::trim)
        .filter(|text| !text.is_empty());
    match (address, name) {
        (Some(address), name) => Some(AddressEntry {
            name,
            address: address.to_string(),
        }),
        (None, Some(text)) => Some(AddressEntry {
            name: None,
            address: text,
        }),
        (None, None) => None,
    }
}

/// Convertit une liste ou un groupe d'adresses en liste plate d'entrées.
pub(crate) fn convert_address(header: Option<&Address<'_>>) -> Vec<AddressEntry> {
    match header {
        Some(Address::List(list)) => list.iter().filter_map(entry_of).collect(),
        Some(Address::Group(groups)) => groups
            .iter()
            .flat_map(|group| group.addresses.iter())
            .filter_map(entry_of)
            .collect(),
        None => Vec::new(),
    }
}
