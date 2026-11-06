//! Messages du démon : lignes fixes sur stdout et stderr, sans secret, et codes de sortie.

use std::io::Write;
use std::process::ExitCode;

/// Écrit une ligne sur stdout et la vide aussitôt (un superviseur la lit en direct).
pub fn say(line: &str) {
    let mut out = std::io::stdout().lock();
    let _ = writeln!(out, "{line}");
    let _ = out.flush();
}

/// Écrit un message fixe sur stderr et rend le code de sortie demandé.
pub fn say_error(message: &str, code: u8) -> ExitCode {
    let _ = writeln!(std::io::stderr().lock(), "mira-daemon : {message}");
    ExitCode::from(code)
}
