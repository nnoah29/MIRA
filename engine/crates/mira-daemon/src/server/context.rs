//! Contexte du serveur : l'hôte, le jeton, les limites, l'état annoncé et le signal d'arrêt.

use std::sync::Arc;
use std::time::Duration;

use mira_host::Host;
use mira_protocol::{HostState, MAX_FRAME_BYTES};
use tokio::sync::{OnceCell, Semaphore, watch};

use super::Clients;
use crate::token::Token;

/// Nombre de connexions qui peuvent attendre leur `hello` en même temps.
const MAX_PENDING_HELLO: usize = 64;

/// Ce que partagent les sessions : l'hôte du moteur, le jeton, les limites et l'état.
#[derive(Debug)]
pub struct ServerContext {
    pub host: Arc<Host>,
    pub token: Token,
    pub max_frame: u32,
    pub hello_timeout: Duration,
    pub event_queue: usize,
    pub event_lag_grace: Duration,
    pub pending_hello: Arc<Semaphore>,
    pub shutdown_wait: Duration,
    pub clients: Clients,
    state: watch::Sender<HostState>,
    closing: watch::Sender<bool>,
    outcome: OnceCell<String>,
}

impl ServerContext {
    /// Contexte aux valeurs par défaut : 16 Mio par trame, `hello` sous 5 s (64 en attente au plus), arrêt patientant 5 s, file d'événements de 256 attendue 5 s avant `lagged`, état `starting`.
    pub fn new(host: Arc<Host>, token: Token) -> Self {
        Self {
            host,
            token,
            max_frame: MAX_FRAME_BYTES,
            hello_timeout: Duration::from_secs(5),
            event_queue: 256,
            event_lag_grace: Duration::from_secs(5),
            pending_hello: Arc::new(Semaphore::new(MAX_PENDING_HELLO)),
            shutdown_wait: Duration::from_secs(5),
            clients: Clients::default(),
            state: watch::channel(HostState::Starting).0,
            closing: watch::channel(false).0,
            outcome: OnceCell::new(),
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

    /// Résultat de la fermeture du moteur, calculé une seule fois même si plusieurs arrêts sont demandés.
    pub fn shutdown_outcome(&self) -> &OnceCell<String> {
        &self.outcome
    }

    /// Déclenche l'arrêt : l'écouteur cesse d'accepter et les sessions se ferment.
    pub fn begin_shutdown(&self) {
        self.closing.send_replace(true);
    }
}
