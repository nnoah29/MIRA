//! Contexte du serveur : l'hôte, le jeton, les limites, l'état annoncé et le signal d'arrêt.

use std::sync::Arc;
use std::time::Duration;

use mira_host::Host;
use mira_protocol::{HostState, MAX_FRAME_BYTES};
use tokio::sync::watch;

use crate::token::Token;

/// Ce que partagent les sessions : l'hôte du moteur, le jeton, les limites et l'état.
#[derive(Debug)]
pub struct ServerContext {
    pub host: Arc<Host>,
    pub token: Token,
    pub max_frame: u32,
    pub hello_timeout: Duration,
    pub event_queue: usize,
    state: watch::Sender<HostState>,
    closing: watch::Sender<bool>,
}

impl ServerContext {
    /// Contexte aux valeurs par défaut : 16 Mio par trame, `hello` sous 5 s, file d'événements de 256, état `starting`.
    pub fn new(host: Arc<Host>, token: Token) -> Self {
        Self {
            host,
            token,
            max_frame: MAX_FRAME_BYTES,
            hello_timeout: Duration::from_secs(5),
            event_queue: 256,
            state: watch::channel(HostState::Starting).0,
            closing: watch::channel(false).0,
        }
    }

    /// Change l'état annoncé aux nouvelles connexions.
    pub fn set_state(&self, state: HostState) {
        self.state.send_replace(state);
    }

    /// État annoncé dans `hello_ok`.
    pub fn host_state(&self) -> HostState {
        *self.state.borrow()
    }

    /// Récepteur du signal d'arrêt : devient vrai quand le serveur doit se fermer.
    pub fn closing_receiver(&self) -> watch::Receiver<bool> {
        self.closing.subscribe()
    }

    /// Déclenche l'arrêt : l'écouteur cesse d'accepter et les sessions se ferment.
    pub fn begin_shutdown(&self) {
        self.closing.send_replace(true);
    }
}

/// Attend le signal d'arrêt ; ne garde aucune référence au canal pendant l'attente (futur `Send`).
pub async fn wait_closing(closing: &mut watch::Receiver<bool>) {
    let _ = closing.wait_for(|closing| *closing).await.map(|_| ());
}
