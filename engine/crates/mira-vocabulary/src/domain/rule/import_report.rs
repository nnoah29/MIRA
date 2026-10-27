//! Rapport d'import d'un fichier de règles : un statut par règle du fichier.

use serde::{Deserialize, Serialize};

/// Raison pour laquelle une règle du fichier n'a pas été importée.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RuleImportRefusal {
    UnknownAccount,
    UnknownMailbox,
    UnknownLabel,
    UnknownTemplate,
    InvalidRule { code: String },
}

/// Sort d'une règle du fichier.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RuleImportOutcome {
    Imported,
    Replaced,
    AlreadyPresent,
    Refused { reason: RuleImportRefusal },
}

/// Ligne du rapport : position (à partir de 1) et nom de la règle dans le fichier, et son sort.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuleImportEntry {
    pub position: u32,
    pub name: String,
    pub outcome: RuleImportOutcome,
}

/// Rapport complet : une ligne par règle du fichier, dans l'ordre du fichier.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuleImportReport {
    pub entries: Vec<RuleImportEntry>,
}
