//! Ouverture du moteur derrière un trousseau verrouillé : état annoncé, réessais espacés ou sur demande, sans fenêtre.

use std::sync::Arc;
use std::time::Duration;

use mira_assembly::api::EngineConfig;
use mira_assembly::mira_vocabulary::SecretLocation;
use mira_assembly::mira_vocabulary::error::codes::secrets;
use mira_assembly::ports::SecretStore;
use mira_protocol::HostState;

use crate::server::{ServerContext, wait_closing};

/// Délais entre deux essais : le premier, puis le double jusqu'au plafond.
#[derive(Debug, Clone, Copy)]
pub struct UnlockBackoff {
    pub first: Duration,
    pub max: Duration,
}

impl Default for UnlockBackoff {
    /// Cinq secondes, doublées jusqu'à une minute.
    fn default() -> Self {
        Self {
            first: Duration::from_secs(5),
            max: Duration::from_secs(60),
        }
    }
}

/// Entrée lue, jamais écrite, pour savoir si le trousseau répond.
struct ProbeKey;

impl SecretLocation for ProbeKey {
    /// Clé fixe, sans rapport avec un compte.
    fn storage_key(&self) -> String {
        "daemon-probe".to_string()
    }
}

/// Dit si le trousseau refuse l'accès parce qu'il est verrouillé.
async fn is_locked(store: &Arc<dyn SecretStore>) -> bool {
    let store = store.clone();
    let probe = tokio::task::spawn_blocking(move || store.get(&ProbeKey)).await;
    matches!(probe, Ok(Err(error)) if error.code == secrets::KEYRING_LOCKED.0)
}

/// Attend le délai, une demande de réessai ou l'arrêt ; vrai si l'arrêt est demandé.
async fn wait_or_stop(ctx: &ServerContext, delay: Duration) -> bool {
    let mut closing = ctx.closing_receiver();
    tokio::select! {
        _ = tokio::time::sleep(delay) => false,
        _ = ctx.retry_unlock.notified() => false,
        _ = wait_closing(&mut closing) => true,
    }
}

/// Ouvre le moteur dès que le trousseau répond ; `None` si l'arrêt est demandé pendant l'attente.
pub async fn open_when_unlocked(
    ctx: &ServerContext,
    engine: EngineConfig,
    backoff: UnlockBackoff,
) -> Option<String> {
    let mut delay = backoff.first;
    while is_locked(&engine.secrets).await {
        ctx.set_state(HostState::KeyringLocked);
        if wait_or_stop(ctx, delay).await {
            return None;
        }
        delay = (delay * 2).min(backoff.max);
    }
    ctx.set_state(HostState::Starting);
    Some(ctx.host.open(engine))
}
