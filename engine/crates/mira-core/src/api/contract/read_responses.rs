//! Formes de réponse des domaines de lecture et de maintenance.

use mira_vocabulary::{MessageSummary, Thread};
use serde::{Deserialize, Serialize};

/// Source brute d'un message, encodée en base64 pour tenir dans du JSON.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RawSourceView {
    pub size: u64,
    pub content_base64: String,
}

/// Un fil et ses messages, du plus ancien au plus récent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThreadView {
    pub thread: Thread,
    pub messages: Vec<MessageSummary>,
}

/// Verdict du contrôle d'intégrité de la base.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntegrityReport {
    pub ok: bool,
}

/// Fichier d'export écrit : chemin demandé et taille en octets.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportReport {
    pub path: String,
    pub bytes: u64,
}
