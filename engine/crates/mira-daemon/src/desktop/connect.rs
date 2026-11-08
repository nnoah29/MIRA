//! Connexion au bureau Linux : bus de session pour les notifications, hôte de zone de notification pour l'icône.

use std::sync::Arc;

use mira_host::notifier::{NotificationSink, Texts};

use super::state_mirror::mirror_state;
use super::{
    DbusNotificationSink, Desktop, Launch, StatusNotifierTray, TrayControl, TrayStatus,
    app_launcher, system_locale,
};
use crate::report::say;
use crate::server::ServerContext;
use crate::shutdown;

impl Desktop {
    /// Se relie au bus de session ; sans bus ou sans hôte de zone de notification, le démon continue sans cette partie.
    pub async fn connect(ctx: &Arc<ServerContext>) -> Self {
        let locale = system_locale();
        let texts = Texts::for_locale(&locale);
        let sink = match DbusNotificationSink::connect(app_launcher()) {
            Ok(sink) => Some(Arc::new(sink) as Arc<dyn NotificationSink>),
            Err(_) => {
                say("notifications du bureau indisponibles");
                None
            }
        };
        let tray = start_tray(ctx, locale).await;
        Self {
            sink,
            _tray: tray.map(|tray| tray as Arc<dyn std::fmt::Debug + Send + Sync>),
            texts,
        }
    }
}

/// Démarre l'icône et la relie à l'état du serveur ; « Quitter » arrête le démon proprement.
async fn start_tray(ctx: &Arc<ServerContext>, locale: String) -> Option<Arc<dyn TrayControl>> {
    let runtime = tokio::runtime::Handle::current();
    let stop = ctx.clone();
    let quit: Launch = Arc::new(move || {
        runtime.spawn(shutdown::run(stop.clone()));
    });
    let started =
        StatusNotifierTray::start(app_launcher(), quit, locale, TrayStatus::Starting).await;
    let Ok(tray) = started else {
        say("icône de zone de notification indisponible");
        return None;
    };
    let tray: Arc<dyn TrayControl> = Arc::new(tray);
    mirror_state(ctx, tray.clone());
    Some(tray)
}
