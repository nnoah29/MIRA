//! Connexion au bureau hors Linux : pas de bus ni d'icône tant que les adaptateurs du système ne sont pas branchés.

use std::sync::Arc;

use mira_host::notifier::{NotificationSink, Texts};

use super::{Desktop, system_locale};
use crate::server::ServerContext;

impl Desktop {
    /// Se relie aux notifications du système quand un adaptateur existe ; sinon le démon continue sans notification ni icône.
    pub async fn connect(ctx: &Arc<ServerContext>) -> Self {
        let locale = system_locale();
        let mut desktop = Self::new(platform_sink(), Texts::for_locale(&locale));
        desktop._tray = platform_tray(ctx, locale);
        desktop
    }
}

/// Icône de la barre des tâches reliée à l'état du serveur ; « Quitter » arrête le démon proprement.
#[cfg(windows)]
fn platform_tray(
    ctx: &Arc<ServerContext>,
    locale: String,
) -> Option<Arc<dyn std::fmt::Debug + Send + Sync>> {
    let runtime = tokio::runtime::Handle::current();
    let stop = ctx.clone();
    let quit: super::Launch = Arc::new(move || {
        runtime.spawn(crate::shutdown::run(stop.clone()));
    });
    let started = super::WindowsTray::start(
        super::app_launcher(),
        quit,
        locale,
        super::TrayStatus::Starting,
    );
    let tray = Arc::new(started.ok()?);
    super::mirror_state(ctx, tray.clone());
    Some(tray)
}

/// Aucune icône sur ce système.
#[cfg(not(windows))]
fn platform_tray(
    _ctx: &Arc<ServerContext>,
    _locale: String,
) -> Option<Arc<dyn std::fmt::Debug + Send + Sync>> {
    None
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
