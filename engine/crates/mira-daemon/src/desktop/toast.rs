//! Récepteur de notifications Windows : toasts du système, identité d'application inscrite pour l'utilisateur, clic qui relance l'app.

use std::sync::Arc;

use mira_host::notifier::{Notification, NotificationSink};
use tauri_winrt_notification::{Sound, Toast};
use winreg::RegKey;
use winreg::enums::HKEY_CURRENT_USER;

use super::Launch;
use super::bound::bound_body;

/// Identité d'application des toasts, la même que celle de l'app Flutter.
pub const APP_USER_MODEL_ID: &str = "com.toffanoah.mira";

/// Affiche les notifications par les toasts de Windows.
pub struct WindowsToastSink {
    launch: Launch,
}

impl std::fmt::Debug for WindowsToastSink {
    /// Rien à montrer
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("WindowsToastSink")
    }
}

/// Inscrit le nom affiché de l'application dans le registre de l'utilisateur ; sans cela Windows n'affiche pas les toasts d'une application non installée.
fn register_app_id() -> std::io::Result<()> {
    let path = format!(r"Software\Classes\AppUserModelId\{APP_USER_MODEL_ID}");
    let (key, _) = RegKey::predef(HKEY_CURRENT_USER).create_subkey(path)?;
    key.set_value("DisplayName", &"MIRA")
}

impl WindowsToastSink {
    /// Inscrit l'identité d'application puis rend le récepteur ; échoue si le registre est inaccessible.
    pub fn connect(launch: Launch) -> std::io::Result<Self> {
        register_app_id()?;
        Ok(Self { launch })
    }
}

impl NotificationSink for WindowsToastSink {
    /// Affiche le toast dans un fil à part ; un échec d'affichage est ignoré, un clic relance l'app.
    fn show(&self, notification: Notification) {
        let launch: Arc<_> = self.launch.clone();
        std::thread::spawn(move || {
            let sound = notification.sound.then_some(Sound::Mail);
            let _ = Toast::new(APP_USER_MODEL_ID)
                .title(&notification.title)
                .text1(&bound_body(&notification.body))
                .sound(sound)
                .on_activated(move |_| {
                    launch();
                    Ok(())
                })
                .show();
        });
    }
}
