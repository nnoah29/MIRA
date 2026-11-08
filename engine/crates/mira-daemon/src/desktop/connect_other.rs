//! Connexion au bureau hors Linux : pas de bus ni d'icône tant que les adaptateurs du système ne sont pas branchés.

use std::sync::Arc;

use mira_host::notifier::{NotificationSink, Texts};

use super::{Desktop, system_locale};
use crate::server::ServerContext;

impl Desktop {
    /// Se relie aux notifications du système quand un adaptateur existe ; sinon le démon continue sans notification ni icône.
    pub async fn connect(_ctx: &Arc<ServerContext>) -> Self {
        Self::new(platform_sink(), Texts::for_locale(&system_locale()))
    }
}

/// Toasts de Windows ; un registre inaccessible laisse le démon sans notification.
#[cfg(windows)]
fn platform_sink() -> Option<Arc<dyn NotificationSink>> {
    let sink = super::WindowsToastSink::connect(super::app_launcher()).ok()?;
    Some(Arc::new(sink))
}

/// Aucun adaptateur de notification sur ce système.
#[cfg(not(windows))]
fn platform_sink() -> Option<Arc<dyn NotificationSink>> {
    None
}
