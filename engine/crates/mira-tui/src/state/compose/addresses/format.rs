//! Écriture des adresses du moteur dans un champ : `a@b.c` ou `Nom <a@b.c>`, noms spéciaux entre guillemets.

use mira_assembly::mira_vocabulary::AddressEntry;

/// Vrai si le nom doit être entouré de guillemets pour être relu à l'identique.
fn needs_quotes(name: &str) -> bool {
    name.contains([',', ';', '<', '>', '"', '\\', '@'])
}

/// Nom prêt à être écrit devant `<adresse>`, entre guillemets si nécessaire.
fn written_name(name: &str) -> String {
    if !needs_quotes(name) {
        return name.to_string();
    }
    let escaped = name.replace('\\', "\\\\").replace('"', "\\\"");
    format!("\"{escaped}\"")
}

/// Une adresse telle qu'on l'écrit dans le champ.
fn format_entry(entry: &AddressEntry) -> String {
    match entry.name.as_deref().map(str::trim) {
        Some(name) if !name.is_empty() => {
            format!("{} <{}>", written_name(name), entry.address)
        }
        Some(_) | None => entry.address.clone(),
    }
}

/// Texte du champ pour ces adresses, séparées par « , ».
pub fn format_addresses(entries: &[AddressEntry]) -> String {
    entries
        .iter()
        .map(format_entry)
        .collect::<Vec<_>>()
        .join(", ")
}
