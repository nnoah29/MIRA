//! Rendu humain des identités d'envoi.

use mira_assembly::mira_vocabulary::Identity;

use super::table::render_table;

/// Cellules d'une ligne d'identité.
fn identity_row(identity: &Identity) -> Vec<String> {
    vec![
        identity.id.to_string(),
        identity.address.as_str().to_string(),
        identity.display_name.clone(),
        if identity.is_default { "oui" } else { "non" }.to_string(),
    ]
}

/// Tableau des identités : identifiant, adresse, nom, par défaut.
pub fn identities_table(identities: &[Identity]) -> String {
    let rows: Vec<Vec<String>> = identities.iter().map(identity_row).collect();
    render_table(&["ID", "ADRESSE", "NOM", "DÉFAUT"], &rows)
}
