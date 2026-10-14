//! Session complète : moteur ouvert, terminal pris, boucle, puis terminal rendu et moteur fermé.

use std::io::stdout;
use std::sync::Arc;

use mira_assembly::api::{Engine, EngineConfig};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use tokio::sync::mpsc;

use super::engine_session::{close_engine, open_engine};
use super::error::TuiError;
use super::input_thread;
use super::run_loop::run_loop;
use super::terminal_guard::TerminalGuard;

/// Ouvre le moteur, fait tourner l'interface, puis ferme le moteur même si l'interface a échoué.
pub async fn run_app(config: EngineConfig) -> Result<(), TuiError> {
    let engine = open_engine(config)?;
    let outcome = run_in_terminal(Arc::clone(&engine)).await;
    let closed = close_engine(engine).await;
    outcome.and(closed)
}

/// Prend le terminal, lance le fil du clavier et la boucle ; le garde rend le terminal en sortant.
async fn run_in_terminal(engine: Arc<Engine>) -> Result<(), TuiError> {
    let _guard = TerminalGuard::enter()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    let (sender, keys) = mpsc::unbounded_channel();
    let reader = input_thread::spawn(sender);
    let outcome = run_loop(&mut terminal, engine, keys).await;
    let _ = terminal.show_cursor();
    let _ = tokio::task::spawn_blocking(move || reader.join()).await;
    outcome
}
