//! Rapport d'import d'un fichier de libellés locaux : un statut par libellé du fichier.

use serde::{Deserialize, Serialize};

/// Raison pour laquelle un libellé du fichier n'a pas été importé.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum LabelImportRefusal {
    InvalidName,
    InvalidColor,
    InvalidLabel,
    DuplicateInFile,
    TooManyLabels,
}

/// Sort d'un libellé du fichier.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum LabelImportOutcome {
    Imported,
    Replaced,
    AlreadyPresent,
    Refused { reason: LabelImportRefusal },
}

/// Ligne du rapport : position (à partir de 1), nom (tronqué) et sort.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LabelImportEntry {
    pub position: u32,
    pub name: String,
    pub outcome: LabelImportOutcome,
}

/// Rapport complet : une ligne par libellé du fichier, dans l'ordre du fichier.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LabelImportReport {
    pub entries: Vec<LabelImportEntry>,
}
