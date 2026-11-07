//! Ce que le démon fait avec le bureau : notifications du moteur, attente du trousseau, langue du système.

use std::sync::Arc;
use std::time::Duration;

use mira_host::Host;
use mira_host::notifier::{Notification, NotificationSink, SystemClock, Texts, run_notifier};

use super::{DbusNotificationSink, app_launcher, system_locale};
use crate::report::say;

const WINDOW: Duration = Duration::from_secs(2);
const KEYRING_NOTICE_ID: u32 = 1;

/// Intégration au bureau d'un démon : un récepteur de notifications s'il y a un bus, et les textes de la langue.
#[derive(Debug)]
pub struct Desktop {
    sink: Option<Arc<dyn NotificationSink>>,
    texts: Texts,
}

impl Desktop {
    /// Intégration avec ce récepteur (ou aucun) et ces textes.
    pub fn new(sink: Option<Arc<dyn NotificationSink>>, texts: Texts) -> Self {
        Self { sink, texts }
    }

    /// Se relie au bus de session ; sans bus, le démon continue sans notifications du bureau.
    pub fn connect() -> Self {
        let texts = Texts::for_locale(&system_locale());
        match DbusNotificationSink::connect(app_launcher()) {
            Ok(sink) => Self::new(Some(Arc::new(sink)), texts),
            Err(_) => {
                say("notifications du bureau indisponibles");
                Self::new(None, texts)
            }
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
