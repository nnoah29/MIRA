//! Fin du démon : ordre d'arrêt, ou inactivité prolongée quand le démon a été lancé à la demande.

use std::time::Duration;

use tokio::time::timeout;

use crate::server::{Clients, ServerContext};
use crate::signals;

/// Rend la main quand aucun client n'est resté attaché pendant toute la durée `grace`.
pub async fn wait_until_idle_for(clients: &Clients, grace: Duration) {
    let mut count = clients.subscribe();
    loop {
        let idle = *count.borrow_and_update() == 0;
        let change = if idle {
            timeout(grace, count.changed()).await
        } else {
            Ok(count.changed().await)
        };
        match change {
            Ok(Ok(())) => {}
            Ok(Err(_)) | Err(_) => return,
        }
    }
}

/// Rend la main à l'ordre d'arrêt ou, si `idle_grace` est donné, à l'inactivité prolongée.
pub async fn wait_for_end(ctx: &ServerContext, idle_grace: Option<Duration>) {
    match idle_grace {
        Some(grace) => tokio::select! {
            _ = signals::wait_for_stop(ctx) => {}
            _ = wait_until_idle_for(&ctx.clients, grace) => {}
        },
        None => signals::wait_for_stop(ctx).await,
    }
}
