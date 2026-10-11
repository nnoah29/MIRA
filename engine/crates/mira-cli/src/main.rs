//! Binaire `mira` : premier client du contrat d'API (12.8).

use std::io::Write;
use std::process::ExitCode;

use mira_cli::exit_codes::INTERNAL;
use mira_cli::{CliIo, run};

/// Point d'entrée du CLI : relaie les arguments et les flux standards à `run`.
fn main() -> ExitCode {
    let args: Vec<String> = std::env::args_os()
        .map(|argument| argument.to_string_lossy().into_owned())
        .collect();
    let mut out = std::io::stdout();
    let mut err = std::io::stderr();
    let code = run(args, &mut CliIo::new(&mut out, &mut err));
    let _ = out.flush();
    ExitCode::from(u8::try_from(code).unwrap_or(INTERNAL as u8))
}
