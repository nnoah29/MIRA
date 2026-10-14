//! Session complète : moteur ouvert, terminal pris, boucle, terminal rendu, avis de fermeture, moteur fermé (AG-39).

use std::io::{Stdout, Write, stderr, stdout};
use std::sync::Arc;

use mira_assembly::api::{Engine, EngineConfig};
use ratatui::Terminal;
use ratatui::backend::{Backend, CrosstermBackend};
use tokio::sync::mpsc::{self, UnboundedReceiver};

use super::engine_session::{close_engine, open_engine};
use super::error::TuiError;
use super::input_thread;
use super::run_loop::{LoopHooks, run_loop_detached};
use super::stop_signals::spawn_stop_signals;
use super::terminal_guard::TerminalGuard;
use crossterm::event::KeyEvent;

const CLOSING_NOTICE: &str = "Fermeture de MIRA…";

/// Ouvre le moteur, prend le terminal, écoute clavier et signaux, puis termine la session ; le moteur est toujours fermé.
pub async fn run_app(config: EngineConfig) -> Result<(), TuiError> {
    let engine = open_engine(config)?;
    let (guard, mut terminal) = match enter_terminal() {
        Ok(parts) => parts,
        Err(error) => {
            let _ = close_engine(engine).await;
            return Err(error);
        }
    };
    let (sender, keys) = mpsc::unbounded_channel();
    let reader = input_thread::spawn(sender);
    let (stop_sender, stop) = mpsc::unbounded_channel();
    let signals = spawn_stop_signals(stop_sender);
    let hooks = LoopHooks {
        frames: None,
        stop: Some(stop),
    };
    let outcome = run_session_parts(engine, guard, &mut terminal, keys, hooks, &mut stderr()).await;
    signals.abort();
    let _ = tokio::task::spawn_blocking(move || reader.join()).await;
    outcome
}

/// Active mode brut et écran alternatif, puis ouvre le terminal ratatui sur la sortie standard.
fn enter_terminal() -> Result<(TerminalGuard, Terminal<CrosstermBackend<Stdout>>), TuiError> {
    let guard = TerminalGuard::enter()?;
    let terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    Ok((guard, terminal))
}

/// Fait tourner la boucle, rend le terminal, écrit l'avis sur `notice`, attend les requêtes en vol puis ferme le moteur.
pub async fn run_session_parts<B: Backend>(
    engine: Arc<Engine>,
    guard: TerminalGuard,
    terminal: &mut Terminal<B>,
    keys: UnboundedReceiver<KeyEvent>,
    hooks: LoopHooks,
    notice: &mut dyn Write,
) -> Result<(), TuiError> {
    let (outcome, stopping) = run_loop_detached(terminal, Arc::clone(&engine), keys, hooks).await;
    let _ = terminal.show_cursor();
    drop(guard);
    let _ = notice.write_all(format!("{CLOSING_NOTICE}\n").as_bytes());
    let _ = stopping.await;
    let closed = close_engine(engine).await;
    outcome.and(closed)
}
