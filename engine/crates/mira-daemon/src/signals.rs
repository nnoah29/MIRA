//! Attente d'un ordre d'arrêt : SIGTERM, SIGINT ou l'opération d'hôte `shutdown`.

use crate::server::{ServerContext, wait_closing};

/// Rend la main quand le démon doit s'arrêter.
pub async fn wait_for_stop(ctx: &ServerContext) {
    let mut closing = ctx.closing_receiver();
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {}
        _ = terminate() => {}
        _ = wait_closing(&mut closing) => {}
    }
}

/// Attend SIGTERM ; sans ce signal (autre système), n'aboutit jamais.
#[cfg(unix)]
async fn terminate() {
    use tokio::signal::unix::{SignalKind, signal};
    match signal(SignalKind::terminate()) {
        Ok(mut stream) => {
            stream.recv().await;
        }
        Err(_) => std::future::pending().await,
    }
}

/// Sans SIGTERM, n'aboutit jamais.
#[cfg(not(unix))]
async fn terminate() {
    std::future::pending::<()>().await
}
