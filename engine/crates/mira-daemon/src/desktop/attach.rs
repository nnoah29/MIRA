//! Ce que le démon fait avec le bureau : notifications du moteur, attente du trousseau, langue du système.

use std::sync::Arc;
use std::time::Duration;

use mira_host::Host;
use mira_host::notifier::{Notification, NotificationSink, SystemClock, Texts, run_notifier};

const WINDOW: Duration = Duration::from_secs(2);
const KEYRING_NOTICE_ID: u32 = 1;
const TEST_NOTICE_ID: u32 = 2;

/// Intégration au bureau d'un démon : un récepteur de notifications s'il y a un bus, et les textes de la langue.
#[derive(Debug)]
pub struct Desktop {
    pub(super) sink: Option<Arc<dyn NotificationSink>>,
    pub(super) _tray: Option<Arc<dyn std::fmt::Debug + Send + Sync>>,
    pub(super) texts: Texts,
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
