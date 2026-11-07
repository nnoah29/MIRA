//! Reflète l'état annoncé par le serveur sur l'icône, tant que le serveur vit.

use std::sync::Arc;

use super::tray::{TrayControl, tray_status_of};
use crate::server::ServerContext;

/// Lance une tâche qui répercute chaque changement d'état de l'hôte sur l'icône.
pub fn mirror_state(ctx: &Arc<ServerContext>, tray: Arc<dyn TrayControl>) {
    let mut states = ctx.state_receiver();
    tray.set_status(tray_status_of(*states.borrow_and_update()));
    tokio::spawn(async move {
        while states.changed().await.is_ok() {
            tray.set_status(tray_status_of(*states.borrow_and_update()));
        }
    });
}
