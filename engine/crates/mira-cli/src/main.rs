//! Binaire `mira` : premier client du contrat d'API (12.8).

use std::io::Write;
use std::process::ExitCode;

use mira_cli::exit_codes::INTERNAL;
use mira_cli::logging::{init_logging, verbosity_from_args};
use mira_cli::{CliDeps, CliIo, run};

/// Point d'entrée du CLI : dépendances réelles, flux standards, exécution asynchrone.
fn main() -> ExitCode {
    let args: Vec<String> = std::env::args_os()
        .map(|argument| argument.to_string_lossy().into_owned())
        .collect();
    init_logging(verbosity_from_args(&args));
    let Ok(runtime) = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
    else {
        return ExitCode::from(INTERNAL as u8);
    };
    let mut deps = CliDeps::production(Box::new(std::io::stdin()));
    let mut out = std::io::stdout();
    let mut err = std::io::stderr();
    let code = runtime.block_on(run(args, &mut CliIo::new(&mut out, &mut err), &mut deps));
    let _ = out.flush();
    ExitCode::from(u8::try_from(code).unwrap_or(INTERNAL as u8))
}
