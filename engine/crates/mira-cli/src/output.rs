//! Écriture du résultat d'une commande, en texte lisible ou en enveloppe JSON.

use mira_assembly::mira_vocabulary::MiraError;
use serde_json::Value;

use crate::cli_io::CliIo;
use crate::exit_codes::{SUCCESS, exit_code_for};
use crate::json_envelope::{error_envelope, success_envelope};

/// Format de sortie demandé par l'utilisateur.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputMode {
    Human,
    Json,
}

impl OutputMode {
    /// Choisit le format d'après la présence de l'option `--json`.
    pub fn from_json_flag(json: bool) -> Self {
        if json { Self::Json } else { Self::Human }
    }
}

/// Écrit un succès sur la sortie standard et rend le code de sortie 0.
pub fn report_success(io: &mut CliIo, mode: OutputMode, data: Value, human: &str) -> i32 {
    let text = match mode {
        OutputMode::Json => success_envelope(data).to_string(),
        OutputMode::Human => human.to_string(),
    };
    let _ = writeln!(io.out, "{text}");
    SUCCESS
}

/// Écrit une erreur (enveloppe sur stdout en JSON, message sur stderr sinon) et rend son code.
pub fn report_error(io: &mut CliIo, mode: OutputMode, error: &MiraError) -> i32 {
    match mode {
        OutputMode::Json => {
            let _ = writeln!(io.out, "{}", error_envelope(error));
        }
        OutputMode::Human => {
            let _ = writeln!(io.err, "erreur : {} ({})", error.message, error.code);
        }
    }
    exit_code_for(error.category)
}
