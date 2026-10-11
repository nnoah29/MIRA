//! Exécution du CLI en processus, avec capture de stdout et de stderr.

use mira_cli::{CliIo, run};

/// Résultat d'une exécution : code de sortie, stdout et stderr.
#[derive(Debug)]
pub struct CliResult {
    pub code: i32,
    pub stdout: String,
    pub stderr: String,
}

/// Lance `mira` avec ces arguments (sans le nom du programme) et capture ses flux.
pub fn run_cli(arguments: &[&str]) -> CliResult {
    let mut args = vec!["mira".to_string()];
    args.extend(arguments.iter().map(ToString::to_string));
    let mut out = Vec::new();
    let mut err = Vec::new();
    let code = run(args, &mut CliIo::new(&mut out, &mut err));
    CliResult {
        code,
        stdout: String::from_utf8_lossy(&out).into_owned(),
        stderr: String::from_utf8_lossy(&err).into_owned(),
    }
}
