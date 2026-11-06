//! Arrêt propre du démon : on arrête d'accepter, on ferme les sessions, on laisse finir les appels puis on ferme le moteur.

use std::sync::Arc;
use std::time::Duration;

use crate::server::ServerContext;

const IDLE_WAIT: Duration = Duration::from_secs(5);
const IDLE_POLL: Duration = Duration::from_millis(20);

/// Déclenche l'arrêt, attend que plus aucun appel ne tienne le moteur, puis ferme le moteur.
pub async fn run(ctx: Arc<ServerContext>) {
    ctx.begin_shutdown();
    wait_until_idle(&ctx).await;
    let _ = ctx.host.close().await;
}

/// Attend, au plus quelques secondes, que le moteur ne soit plus tenu que par l'hôte.
async fn wait_until_idle(ctx: &ServerContext) {
    let deadline = tokio::time::Instant::now() + IDLE_WAIT;
    while tokio::time::Instant::now() < deadline {
        let Some(engine) = ctx.host.engine() else {
            return;
        };
        if Arc::strong_count(&engine) <= 2 {
            return;
        }
        drop(engine);
        tokio::time::sleep(IDLE_POLL).await;
    }
}
