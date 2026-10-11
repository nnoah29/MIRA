//! `mira provider list` : catalogue des providers, le repli en dernier.

use mira_assembly::ProviderRegistry;
use mira_assembly::mira_vocabulary::{MiraError, ProviderDescriptor};
use serde_json::Value;

use super::labels::auth_list_label;
use crate::cli_io::CliIo;
use crate::json_envelope::{object, to_json_value};
use crate::output::{OutputMode, report_error, report_success};

/// Écrit une ligne lisible : identifiant, nom, domaines et authentifications.
fn describe_row(descriptor: &ProviderDescriptor) -> String {
    let domains = if descriptor.domains.is_empty() {
        "(repli : tout autre domaine)".to_string()
    } else {
        descriptor.domains.join(", ")
    };
    let auth = auth_list_label(&descriptor.auth_methods);
    format!(
        "{:<8} {:<8} auth: {:<24} domaines: {}",
        descriptor.id.as_str(),
        descriptor.display_name,
        auth,
        domains
    )
}

/// Prépare les données JSON et le texte lisible du catalogue.
fn list_output(registry: &ProviderRegistry) -> Result<(Value, String), MiraError> {
    let descriptors = registry.list();
    let data = object(vec![("providers", to_json_value(&descriptors)?)]);
    let rows: Vec<String> = descriptors.iter().map(describe_row).collect();
    Ok((data, rows.join("\n")))
}

/// Exécute `mira provider list`.
pub fn run_list(registry: &ProviderRegistry, mode: OutputMode, io: &mut CliIo) -> i32 {
    match list_output(registry) {
        Ok((data, human)) => report_success(io, mode, data, &human),
        Err(error) => report_error(io, mode, &error),
    }
}
