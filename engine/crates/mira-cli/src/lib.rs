//! Bibliothèque du client `mira` : analyse des arguments, commandes par domaine, sortie versionnée.
//! Le binaire se limite à appeler `run` (testable en processus).

pub mod arguments;
pub mod cli_io;
pub mod commands;
pub mod exit_codes;
pub mod json_envelope;
pub mod output;
pub mod usage;

use clap::Parser;

pub use cli_io::CliIo;

/// Exécute le CLI avec ses arguments (nom du programme compris) et rend le code de sortie.
pub fn run(args: Vec<String>, io: &mut CliIo) -> i32 {
    match arguments::Cli::try_parse_from(&args) {
        Ok(cli) => commands::dispatch(cli, io),
        Err(error) => usage::report_usage_error(&error, &args, io),
    }
}
