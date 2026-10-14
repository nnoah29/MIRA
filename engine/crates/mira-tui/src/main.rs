//! Binaire `mira-tui` : client plein écran de MIRA.

use std::process::ExitCode;

use mira_assembly::mira_vocabulary::ProfileId;
use mira_assembly::production_config;
use mira_tui::app::panic_hook::install_panic_hook;
use mira_tui::app::session::run_app;
use tracing_subscriber::EnvFilter;

/// Installe la journalisation sur stderr, désactivée sauf si `MIRA_LOG` la demande.
fn init_logging() {
    let filter = EnvFilter::try_from_env("MIRA_LOG").unwrap_or_else(|_| EnvFilter::new("off"));
    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .try_init();
}

/// Point d'entrée : session sur le profil par défaut ; l'erreur s'affiche une fois le terminal rendu.
fn main() -> ExitCode {
    init_logging();
    install_panic_hook();
    let runtime = match tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
    {
        Ok(runtime) => runtime,
        Err(error) => return report(&error),
    };
    let config = production_config(ProfileId::from_raw("default"));
    match runtime.block_on(run_app(config)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => report(&error),
    }
}

/// Affiche l'erreur (déjà expurgée de tout secret par le moteur) et rend le code d'échec.
fn report(error: &dyn std::fmt::Display) -> ExitCode {
    eprintln!("mira-tui : {error}");
    ExitCode::FAILURE
}
