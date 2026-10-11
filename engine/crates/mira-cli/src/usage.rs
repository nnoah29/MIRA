//! Erreurs d'usage de la ligne de commande : aide, version et arguments invalides.

use mira_assembly::mira_vocabulary::error::codes::invalid_input;
use mira_assembly::mira_vocabulary::{ErrorCategory, MiraError};

use crate::cli_io::CliIo;
use crate::exit_codes::SUCCESS;
use crate::output::{OutputMode, report_error};

/// Traite le refus d'analyse de clap : aide et version sortent en 0, le reste est `invalid_input`.
pub fn report_usage_error(error: &clap::Error, args: &[String], io: &mut CliIo) -> i32 {
    if !error.use_stderr() {
        let _ = write!(io.out, "{}", error.render());
        return SUCCESS;
    }
    let message = error.to_string();
    let first_line = message.lines().next().unwrap_or("usage invalide");
    let failure = MiraError::new(
        ErrorCategory::InvalidInput,
        invalid_input::PARAMETER,
        first_line.trim_start_matches("error: "),
    );
    let json = args.iter().any(|argument| argument == "--json");
    report_error(io, OutputMode::from_json_flag(json), &failure)
}
