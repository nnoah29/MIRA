//! `mira provider detect <adresse>` : provider reconnu et serveurs à utiliser.

use mira_assembly::ProviderRegistry;
use mira_assembly::mira_vocabulary::{EmailAddress, MiraError, ProviderDescriptor, ServerSettings};
use serde_json::Value;

use super::labels::{auth_list_label, endpoint_label};
use crate::cli_io::CliIo;
use crate::json_envelope::{object, to_json_value};
use crate::output::{OutputMode, report_error, report_success};

/// Écrit le résumé lisible : provider, serveurs et authentifications.
fn human_summary(descriptor: &ProviderDescriptor, servers: &ServerSettings) -> String {
    format!(
        "Provider : {} ({})\nIMAP     : {}\nSMTP     : {}\nAuth     : {}",
        descriptor.id.as_str(),
        descriptor.display_name,
        endpoint_label(&servers.imap),
        endpoint_label(&servers.smtp),
        auth_list_label(&descriptor.auth_methods)
    )
}

/// Analyse l'adresse puis prépare les données JSON et le texte lisible.
fn detect_output(registry: &ProviderRegistry, text: &str) -> Result<(Value, String), MiraError> {
    let address = EmailAddress::parse(text)?;
    let provider = registry.detect(&address);
    let descriptor = provider.descriptor();
    let servers = provider.servers_for(&address);
    let data = object(vec![
        ("address", Value::String(address.as_str().to_string())),
        ("provider", to_json_value(&descriptor)?),
        ("servers", to_json_value(&servers)?),
    ]);
    Ok((data, human_summary(&descriptor, &servers)))
}

/// Exécute `mira provider detect <adresse>`.
pub fn run_detect(
    registry: &ProviderRegistry,
    address: &str,
    mode: OutputMode,
    io: &mut CliIo,
) -> i32 {
    match detect_output(registry, address) {
        Ok((data, human)) => report_success(io, mode, data, &human),
        Err(error) => report_error(io, mode, &error),
    }
}
