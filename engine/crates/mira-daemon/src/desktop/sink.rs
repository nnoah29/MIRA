//! Récepteur de notifications par D-Bus : affiche, retient les numéros donnés par le service, relance l'app au clic.

use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use mira_host::notifier::{Notification, NotificationSink};

use super::Launch;
use super::hints::{action_label, body_text, hints_for};
use super::locale::system_locale;
use super::proxy::NotificationsProxyBlocking;

const APP_NAME: &str = "MIRA";
const APP_ICON: &str = "io.github.nnoah29.Mira";

/// Le bus de session est inutilisable.
#[derive(Debug, thiserror::Error)]
pub enum DesktopError {
    #[error("bus de session indisponible")]
    Bus,
}

/// Affiche les notifications par `org.freedesktop.Notifications` et écoute les clics sur les siennes.
#[derive(Debug)]
pub struct DbusNotificationSink {
    proxy: NotificationsProxyBlocking<'static>,
    owned: Arc<Mutex<HashSet<u32>>>,
    markup: bool,
}

impl DbusNotificationSink {
    /// Se connecte au bus de session de l'utilisateur.
    pub fn connect(launch: Launch) -> Result<Self, DesktopError> {
        let connection = zbus::blocking::Connection::session().map_err(|_| DesktopError::Bus)?;
        Self::over(connection, launch)
    }

    /// Se connecte au bus à cette adresse (tests, bus privé).
    pub fn connect_to(address: &str, launch: Launch) -> Result<Self, DesktopError> {
        let built = zbus::blocking::connection::Builder::address(address).and_then(|b| b.build());
        Self::over(built.map_err(|_| DesktopError::Bus)?, launch)
    }

    /// Prépare le mandataire D-Bus et lance l'écoute des clics.
    fn over(connection: zbus::blocking::Connection, launch: Launch) -> Result<Self, DesktopError> {
        let proxy = NotificationsProxyBlocking::new(&connection).map_err(|_| DesktopError::Bus)?;
        let owned = Arc::new(Mutex::new(HashSet::new()));
        let clicks = proxy
            .receive_action_invoked()
            .map_err(|_| DesktopError::Bus)?;
        let mine = owned.clone();
        std::thread::spawn(move || {
            for click in clicks {
                let Ok(args) = click.args() else { continue };
                let known = mine.lock().is_ok_and(|ids| ids.contains(&args.id));
                if known && args.action_key == "default" {
                    launch();
                }
            }
        });
        let markup = proxy
            .get_capabilities()
            .is_ok_and(|caps| caps.iter().any(|cap| cap == "body-markup"));
        Ok(Self {
            proxy,
            owned,
            markup,
        })
    }
}

impl NotificationSink for DbusNotificationSink {
    /// Envoie la notification au service dans un fil à part ; un service absent ne fait rien.
    fn show(&self, notification: Notification) {
        let proxy = self.proxy.clone();
        let owned = self.owned.clone();
        let markup = self.markup;
        std::thread::spawn(move || {
            let label = action_label(&system_locale());
            let sent = proxy.notify(
                APP_NAME,
                0,
                APP_ICON,
                &notification.title,
                &body_text(&notification.body, markup),
                &["default", label],
                hints_for(notification.sound),
                -1,
            );
            if let (Ok(id), Ok(mut ids)) = (sent, owned.lock()) {
                ids.insert(id);
            }
        });
    }
}
