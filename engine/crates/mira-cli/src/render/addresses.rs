//! Rendu humain des adresses d'en-tête : `Nom <adresse>` ou l'adresse seule.

use mira_assembly::mira_vocabulary::AddressEntry;

/// Une adresse, avec son nom d'affichage s'il existe.
pub fn address_label(entry: &AddressEntry) -> String {
    match entry.name.as_deref().filter(|name| !name.is_empty()) {
        Some(name) => format!("{name} <{}>", entry.address),
        None => entry.address.clone(),
    }
}

/// Plusieurs adresses séparées par une virgule, ou un tiret s'il n'y en a pas.
pub fn address_list(entries: &[AddressEntry]) -> String {
    if entries.is_empty() {
        return "-".to_string();
    }
    let labels: Vec<String> = entries.iter().map(address_label).collect();
    labels.join(", ")
}

/// Nom ou adresse du premier correspondant, pour les colonnes étroites des tableaux.
pub fn first_party(entries: &[AddressEntry]) -> String {
    match entries.first() {
        Some(entry) => entry
            .name
            .clone()
            .filter(|name| !name.is_empty())
            .unwrap_or_else(|| entry.address.clone()),
        None => "-".to_string(),
    }
}
