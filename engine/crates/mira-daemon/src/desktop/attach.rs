//! Ce que le démon fait avec le bureau : notifications du moteur, attente du trousseau, langue du système.

use std::sync::Arc;
use std::time::Duration;

use mira_host::Host;
use mira_host::notifier::{Notification, NotificationSink, SystemClock, Texts, run_notifier};

use super::state_mirror::mirror_state;
use super::{
    DbusNotificationSink, Launch, StatusNotifierTray, TrayControl, TrayStatus, app_launcher,
    system_locale,
};
use crate::report::say;
use crate::server::ServerContext;
use crate::shutdown;

const WINDOW: Duration = Duration::from_secs(2);
const KEYRING_NOTICE_ID: u32 = 1;
const TEST_NOTICE_ID: u32 = 2;

/// Intégration au bureau d'un démon : un récepteur de notifications s'il y a un bus, et les textes de la langue.
#[derive(Debug)]
pub struct Desktop {
    sink: Option<Arc<dyn NotificationSink>>,
    _tray: Option<Arc<dyn TrayControl>>,
    texts: Texts,
}

impl Desktop {
    /// Intégration avec ce récepteur (ou aucun) et ces textes.
    pub fn new(sink: Option<Arc<dyn NotificationSink>>, texts: Texts) -> Self {
        Self {
            sink,
            _tray: None,
            texts,
        }
    }

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
            _tray: tray,
            texts,
        }
    }

    /// Signale, sans son, que le démon attend le déverrouillage du trousseau.
    pub fn announce_keyring_locked(&self) {
        let Some(sink) = &self.sink else { return };
        sink.show(Notification {
            id: KEYRING_NOTICE_ID,
            title: self.texts.keyring_locked_title.clone(),
            body: self.texts.keyring_locked_body.clone(),
            sound: false,
            message: None,
        });
    }

    /// Affiche une notification d'essai, pour que l'utilisateur vérifie que le démon sait notifier.
    pub fn announce_test(&self) {
        let Some(sink) = &self.sink else { return };
        sink.show(Notification {
            id: TEST_NOTICE_ID,
            title: "MIRA".to_string(),
            body: self.texts.test_body.clone(),
            sound: true,
            message: None,
        });
    }

    /// Lance le notifieur du moteur, qui s'arrête avec le moteur.
    pub fn start_notifier(&self, host: Arc<Host>) {
        let Some(sink) = &self.sink else { return };
        let clock = Arc::new(SystemClock);
        tokio::spawn(run_notifier(
            host,
            sink.clone(),
            clock,
            self.texts.clone(),
            WINDOW,
        ));
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
