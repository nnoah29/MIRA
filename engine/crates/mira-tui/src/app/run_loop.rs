//! Boucle d'application : touches et messages du pont → `update` → commandes, puis redessin.

use std::sync::Arc;
use std::time::Duration;

use crossterm::event::KeyEvent;
use mira_assembly::api::Engine;
use ratatui::Terminal;
use ratatui::backend::Backend;
use ratatui::buffer::Buffer;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};

use super::error::TuiError;
use super::runtime::{Runtime, Wake};
use super::screen::{Screen, terminal_error};
use super::session_nonce::session_nonce;
use super::stop_signals::spawn_stop_signals;
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

/// Fait tourner l'interface jusqu'à `q`, un signal d'arrêt ou la fin du clavier ; ne garde aucun clone du moteur.
pub async fn run_loop<B: Backend>(
    terminal: &mut Terminal<B>,
    engine: Arc<Engine>,
    keys: UnboundedReceiver<KeyEvent>,
) -> Result<(), TuiError> {
    let (sender, stop) = mpsc::unbounded_channel();
    let signals = spawn_stop_signals(sender);
    let hooks = LoopHooks {
        frames: None,
        stop: Some(stop),
    };
    let outcome = run_loop_with(terminal, engine, keys, hooks).await;
    signals.abort();
    outcome
}

/// Même boucle avec des options injectées (images copiées, demandes d'arrêt).
pub async fn run_loop_with<B: Backend>(
    terminal: &mut Terminal<B>,
    engine: Arc<Engine>,
    keys: UnboundedReceiver<KeyEvent>,
    hooks: LoopHooks,
) -> Result<(), TuiError> {
    let screen = Screen {
        terminal,
        frames: hooks.frames,
    };
    run_session(screen, engine, keys, hooks.stop).await
}

/// Crée l'état avec un nonce neuf, lance démarrage et flux d'événements, tourne, puis arrête le pont.
async fn run_session<B: Backend>(
    mut screen: Screen<'_, B>,
    engine: Arc<Engine>,
    keys: UnboundedReceiver<KeyEvent>,
    stop: Option<UnboundedReceiver<()>>,
) -> Result<(), TuiError> {
    let size = screen.terminal.size().map_err(terminal_error)?;
    let mut state = AppState::new(size.width, size.height, session_nonce());
    let mut runtime = Runtime::start(engine);
    let mut startup = state.boot_commands();
    startup.extend(refresh_status(&mut state));
    runtime.run(startup);
    let outcome = drive(&mut screen, &mut state, &mut runtime, keys, stop).await;
    runtime.stop().await;
    outcome
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
