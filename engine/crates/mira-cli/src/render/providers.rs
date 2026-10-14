//! Rendu humain des providers et de la préparation d'un compte.

use mira_assembly::api::contract::AccountPreparation;
use mira_assembly::mira_vocabulary::ProviderDescriptor;

use super::table::render_table;
use crate::commands::provider::labels::{auth_list_label, endpoint_label};

/// Cellules d'une ligne de provider.
fn provider_row(descriptor: &ProviderDescriptor) -> Vec<String> {
    vec![
        descriptor.id.as_str().to_string(),
        descriptor.display_name.clone(),
        auth_list_label(&descriptor.auth_methods),
        descriptor.domains.join(", "),
    ]
}

/// Tableau des providers : identifiant, nom, authentifications, domaines.
pub fn providers_table(descriptors: &[ProviderDescriptor]) -> String {
    let rows: Vec<Vec<String>> = descriptors.iter().map(provider_row).collect();
    render_table(&["ID", "NOM", "AUTH", "DOMAINES"], &rows)
}

/// Provider retenu et serveurs proposés pour une adresse.
pub fn preparation_text(preparation: &AccountPreparation) -> String {
    format!(
        "Provider : {} ({})\nIMAP     : {}\nSMTP     : {}\nAuth     : {}",
        preparation.provider.id.as_str(),
        preparation.provider.display_name,
        endpoint_label(&preparation.servers.imap),
        endpoint_label(&preparation.servers.smtp),
        auth_list_label(&preparation.provider.auth_methods)
    )
}
