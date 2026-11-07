//! Ouverture du moteur derrière un trousseau verrouillé : état annoncé, réessais espacés ou sur demande, sans fenêtre.

use std::time::Duration;

use mira_assembly::api::EngineConfig;
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

/// Sonde de l'état du trousseau du système.
pub trait KeyringProbe: Send + Sync + std::fmt::Debug {
    /// Vrai si le trousseau est verrouillé ; appel bloquant, jamais d'invite de déverrouillage.
    fn locked(&self) -> bool;
}

/// Trousseau qui ne se verrouille jamais (trousseau chiffré par fichier).
#[derive(Debug, Clone, Copy)]
pub struct NeverLocked;

impl KeyringProbe for NeverLocked {
    /// Jamais verrouillé.
    fn locked(&self) -> bool {
        false
    }
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

/// Ouvre le moteur dès que le trousseau répond ; `on_locked` est appelé une seule fois, au premier refus ; `None` si l'arrêt est demandé pendant l'attente.
pub async fn open_when_unlocked(
    ctx: &ServerContext,
    engine: EngineConfig,
    backoff: UnlockBackoff,
    probe: &dyn KeyringProbe,
    on_locked: impl FnOnce(),
) -> Option<String> {
    let mut delay = backoff.first;
    let mut on_locked = Some(on_locked);
    while tokio::task::block_in_place(|| probe.locked()) {
        ctx.set_state(HostState::KeyringLocked);
        if let Some(announce) = on_locked.take() {
            announce();
        }
        if wait_or_stop(ctx, delay).await {
            return None;
        }
        delay = (delay * 2).min(backoff.max);
    }
    ctx.set_state(HostState::Starting);
    Some(ctx.host.open(engine))
}
