//! Boucle d'application : touches et messages du pont → `update` → commandes, puis redessin.

use std::sync::Arc;
use std::time::Duration;

use crossterm::event::KeyEvent;
use mira_assembly::api::Engine;
use ratatui::Terminal;
use ratatui::backend::Backend;
use ratatui::buffer::Buffer;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::task::JoinHandle;

use super::error::TuiError;
use super::runtime::{Runtime, Wake};
use super::screen::{Screen, terminal_error};
use super::session_nonce::session_nonce;
use crate::input::intent_for;
use crate::state::AppState;
use crate::update::sync_status::refresh_status;
use crate::update::{Msg, update};

const TICK: Duration = Duration::from_millis(200);

/// Options de la boucle : copie de chaque image dessinée (tests) et source de demandes d'arrêt (signaux).
#[derive(Debug, Default)]
pub struct LoopHooks {
    pub frames: Option<UnboundedSender<Buffer>>,
    pub stop: Option<UnboundedReceiver<()>>,
}

/// Même boucle avec des options injectées (images copiées, demandes d'arrêt).
pub async fn run_loop_with<B: Backend>(
    terminal: &mut Terminal<B>,
    engine: Arc<Engine>,
    keys: UnboundedReceiver<KeyEvent>,
    hooks: LoopHooks,
) -> Result<(), TuiError> {
    let (outcome, stopping) = run_loop_detached(terminal, engine, keys, hooks).await;
    let _ = stopping.await;
    outcome
}

/// Même boucle, sans attendre la fin des requêtes en vol : la poignée rendue finit quand le moteur n'est plus tenu.
pub async fn run_loop_detached<B: Backend>(
    terminal: &mut Terminal<B>,
    engine: Arc<Engine>,
    keys: UnboundedReceiver<KeyEvent>,
    hooks: LoopHooks,
) -> (Result<(), TuiError>, JoinHandle<()>) {
    let screen = Screen {
        terminal,
        frames: hooks.frames,
    };
    run_session(screen, engine, keys, hooks.stop).await
}

/// Crée l'état avec un nonce neuf, lance démarrage et flux d'événements, tourne, puis lance l'arrêt du pont.
async fn run_session<B: Backend>(
    mut screen: Screen<'_, B>,
    engine: Arc<Engine>,
    keys: UnboundedReceiver<KeyEvent>,
    stop: Option<UnboundedReceiver<()>>,
) -> (Result<(), TuiError>, JoinHandle<()>) {
    let size = match screen.terminal.size().map_err(terminal_error) {
        Ok(size) => size,
        Err(error) => return (Err(error), tokio::spawn(async {})),
    };
    let mut state = AppState::new(size.width, size.height, session_nonce());
    let mut runtime = Runtime::start(engine);
    let mut startup = state.boot_commands();
    startup.extend(refresh_status(&mut state));
    runtime.run(startup);
    let outcome = drive(&mut screen, &mut state, &mut runtime, keys, stop).await;
    (outcome, runtime.stop_detached())
}

/// Tour de boucle : taille, dessin, attente d'un réveil, transition ; s'arrête sur sortie, signal ou clavier fermé.
async fn drive<B: Backend>(
    screen: &mut Screen<'_, B>,
    state: &mut AppState,
    runtime: &mut Runtime,
    mut keys: UnboundedReceiver<KeyEvent>,
    mut stop: Option<UnboundedReceiver<()>>,
) -> Result<(), TuiError> {
    let mut tick = tokio::time::interval(TICK);
    loop {
        if let Some(resize) = screen.resized(state)? {
            apply(state, runtime, resize);
        }
        screen.draw(state)?;
        let msg = match runtime.next_wake(&mut keys, &mut stop, &mut tick).await {
            Wake::Key(key) => intent_for(state, key).map(Msg::Intent),
            Wake::Msg(msg) => Some(msg),
            Wake::Settled | Wake::Tick => None,
            Wake::Stop | Wake::KeysClosed => return Ok(()),
        };
        if msg.is_some_and(|msg| apply(state, runtime, msg)) || apply_pending(state, runtime) {
            return Ok(());
        }
    }
}

/// Applique le message et lance ses commandes ; vrai si l'application doit s'arrêter.
fn apply(state: &mut AppState, runtime: &mut Runtime, msg: Msg) -> bool {
    let commands = update(state, msg);
    let quit = runtime.run(commands);
    quit || state.should_quit
}

/// Applique tous les messages déjà arrivés avant de redessiner : un dessin par rafale ; vrai si l'application doit s'arrêter.
fn apply_pending(state: &mut AppState, runtime: &mut Runtime) -> bool {
    while let Some(msg) = runtime.pending_msg() {
        if apply(state, runtime, msg) {
            return true;
        }
    }
    false
}
