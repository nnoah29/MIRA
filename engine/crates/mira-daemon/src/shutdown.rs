//! Arrêt propre du démon : on arrête d'accepter, on ferme les sessions, on laisse finir les appels puis on ferme le moteur.

use std::sync::Arc;
use std::time::Duration;

use crate::server::ServerContext;

const IDLE_POLL: Duration = Duration::from_millis(20);

/// Arrête le démon et rend l'enveloppe de la fermeture du moteur ; plusieurs appels simultanés attendent le même arrêt.
pub async fn run(ctx: Arc<ServerContext>) -> String {
    ctx.shutdown_outcome()
        .get_or_init(|| close_engine(&ctx))
        .await
        .clone()
}

/// Déclenche l'arrêt, attend que plus aucun appel ne tienne le moteur, puis ferme le moteur.
async fn close_engine(ctx: &ServerContext) -> String {
    ctx.begin_shutdown();
    wait_until_idle(ctx).await;
    ctx.host.close().await
}

/// Attend, au plus `shutdown_wait`, que le moteur ne soit plus tenu que par l'hôte.
async fn wait_until_idle(ctx: &ServerContext) {
    let deadline = tokio::time::Instant::now() + ctx.shutdown_wait;
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
