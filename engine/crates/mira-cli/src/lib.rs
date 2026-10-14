//! Bibliothèque du client `mira` : analyse des arguments, commandes par domaine, sortie versionnée.
//! Le binaire se limite à appeler `run` (testable en processus avec des dépendances injectées).

pub mod argument_context;
pub mod argument_error;
pub mod arguments;
pub mod civil_date;
pub mod cli_io;
pub mod commands;
pub mod deps;
pub mod exit_codes;
pub mod json_envelope;
pub mod logging;
pub mod output;
pub mod render;
pub mod usage;

use clap::Parser;

pub use cli_io::CliIo;
pub use deps::CliDeps;

/// Exécute le CLI avec ses arguments (nom du programme compris) et rend le code de sortie.
pub async fn run(args: Vec<String>, io: &mut CliIo<'_>, deps: &mut CliDeps) -> i32 {
    match arguments::Cli::try_parse_from(&args) {
        Ok(cli) => commands::dispatch(cli, io, deps).await,
        Err(error) => usage::report_usage_error(&error, &args, io),
    }
}
