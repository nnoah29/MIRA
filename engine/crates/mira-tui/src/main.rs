//! Binaire `mira-tui` : client plein écran de MIRA.

use std::process::ExitCode;

use tracing_subscriber::EnvFilter;

/// Installe la journalisation sur stderr, désactivée sauf si `MIRA_LOG` la demande.
fn init_logging() {
    let filter = EnvFilter::try_from_env("MIRA_LOG").unwrap_or_else(|_| EnvFilter::new("off"));
    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .try_init();
}

/// Point d'entrée du client plein écran.
fn main() -> ExitCode {
    init_logging();
    ExitCode::SUCCESS
}
