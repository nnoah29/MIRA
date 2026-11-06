//! Enveloppe JSON versionnée échangée avec Flutter (même forme que `mira --json`, invariant 33).

use mira_assembly::mira_vocabulary::error::codes::internal;
use mira_assembly::mira_vocabulary::{CONTRACT_VERSION, ErrorCategory, MiraError};
use serde_json::{Value, json};

/// Enveloppe de succès : version du contrat, `ok` vrai et les données.
pub fn success(data: Value) -> String {
    json!({ "contract_version": CONTRACT_VERSION, "ok": true, "data": data }).to_string()
}

/// Enveloppe d'échec : version du contrat, `ok` faux et l'erreur structurée du moteur.
pub fn failure(error: &MiraError) -> String {
    json!({ "contract_version": CONTRACT_VERSION, "ok": false, "error": error }).to_string()
}

/// Enveloppe d'échec pour une erreur interne du pont (JSON illisible, moteur fermé).
pub fn internal_failure(message: &str) -> String {
    failure(&MiraError::new(
        ErrorCategory::Internal,
        internal::BUG,
        message,
    ))
}
