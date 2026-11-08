//! Démarrage de l'icône auprès de l'hôte de zone de notification et mise à jour de son état.

use ksni::{Handle, TrayMethods};

use super::control::{TrayControl, TrayError};
use super::item::MiraTray;
use super::status::TrayStatus;
use crate::desktop::Launch;

/// Icône StatusNotifierItem en marche.
pub struct StatusNotifierTray {
    handle: Handle<MiraTray>,
}

impl std::fmt::Debug for StatusNotifierTray {
    /// Ne montre rien de l'icône.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("StatusNotifierTray")
    }
}

impl StatusNotifierTray {
    /// Enregistre l'icône ; `launch` relance l'app, `quit` arrête le démon.
    pub async fn start(
        launch: Launch,
        quit: Launch,
        locale: String,
        status: TrayStatus,
    ) -> Result<Self, TrayError> {
        let tray = MiraTray {
            status,
            locale,
            launch,
            quit,
        };
        let handle = tray.spawn().await.map_err(|_| TrayError::NoHost)?;
        Ok(Self { handle })
    }
}

impl TrayControl for StatusNotifierTray {
    /// Met l'état à jour dans une tâche à part (l'appelant n'attend pas le bus).
    fn set_status(&self, status: TrayStatus) {
        let handle = self.handle.clone();
        tokio::spawn(async move {
            let _ = handle.update(|tray| tray.status = status).await;
        });
    }
}
