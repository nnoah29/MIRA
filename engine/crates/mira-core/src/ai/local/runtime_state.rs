//! État du serveur en marche : réutilisation, arrêt et surveillance de l'inactivité.

use std::sync::Arc;
use std::time::Duration;

use tokio::sync::Mutex;

use super::process::terminate;
use super::running::Running;
use super::runtime::LocalRuntimeHandle;

/// État partagé : le serveur en marche et un compteur qui invalide les anciens minuteurs d'inactivité.
#[derive(Default)]
pub(super) struct State {
    pub(super) running: Option<Running>,
    pub(super) generation: u64,
}

/// Réutilise le serveur du même modèle s'il vit ; un processus sorti est oublié.
pub(super) fn reuse(state: &mut State, model_id: &str) -> Option<LocalRuntimeHandle> {
    let running = state.running.as_mut()?;
    if !matches!(running.child.try_wait(), Ok(None)) {
        state.running = None;
        return None;
    }
    if running.model_id != model_id {
        return None;
    }
    running.touch();
    Some(running.handle())
}

/// Arrête et récolte le serveur en marche, s'il y en a un.
pub(super) async fn stop_locked(state: &mut State) {
    if let Some(mut running) = state.running.take() {
        terminate(&mut running.child).await;
    }
}

/// Surveille l'inactivité : arrête le serveur de cette génération quand il n'est plus sollicité.
pub(super) fn spawn_idle_monitor(inner: Arc<Mutex<State>>, generation: u64, idle: Duration) {
    let poll = (idle / 4).clamp(Duration::from_millis(50), Duration::from_secs(1));
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(poll).await;
            let mut state = inner.lock().await;
            let idle_for = match state.running.as_ref() {
                Some(running) if state.generation == generation => running.idle_for(),
                _ => return,
            };
            if idle_for >= idle {
                stop_locked(&mut state).await;
                return;
            }
        }
    });
}
