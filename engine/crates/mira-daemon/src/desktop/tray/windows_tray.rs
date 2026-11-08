//! Icône de la zone de notification de Windows : « Ouvrir MIRA », la ligne d'état, « Quitter MIRA ».

use std::sync::Mutex;

use tray_item::{IconSource, TIError, TrayItem};

use super::control::{TrayControl, TrayError};
use super::status::{TrayStatus, menu_labels, status_line};
use crate::desktop::Launch;

const ICON_RESOURCE: &str = "MIRA_TRAY";

/// Icône de la barre des tâches en marche.
pub struct WindowsTray {
    item: Mutex<TrayItem>,
    status_id: u32,
    locale: String,
}

impl std::fmt::Debug for WindowsTray {
    /// Ne montre rien de l'icône.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("WindowsTray")
    }
}

/// Ajoute les trois entrées et rend le numéro de la ligne d'état.
fn fill_menu(
    item: &mut TrayItem,
    (open, quit): (Launch, Launch),
    locale: &str,
    status: TrayStatus,
) -> Result<u32, TIError> {
    let (open_label, quit_label) = menu_labels(locale);
    item.add_menu_item(open_label, move || open())?;
    let status_id = item
        .inner_mut()
        .add_label_with_id(status_line(status, locale))?;
    item.add_menu_item(quit_label, move || quit())?;
    Ok(status_id)
}

impl WindowsTray {
    /// Crée l'icône ; `launch` relance l'app, `quit` arrête le démon.
    pub fn start(
        launch: Launch,
        quit: Launch,
        locale: String,
        status: TrayStatus,
    ) -> Result<Self, TrayError> {
        let mut item = TrayItem::new("MIRA", IconSource::Resource(ICON_RESOURCE))
            .map_err(|_| TrayError::NoHost)?;
        let status_id =
            fill_menu(&mut item, (launch, quit), &locale, status).map_err(|_| TrayError::NoHost)?;
        Ok(Self {
            item: Mutex::new(item),
            status_id,
            locale,
        })
    }
}

impl TrayControl for WindowsTray {
    /// Remplace le texte de la ligne d'état.
    fn set_status(&self, status: TrayStatus) {
        let Ok(mut item) = self.item.lock() else {
            return;
        };
        let _ = item
            .inner_mut()
            .set_label(status_line(status, &self.locale), self.status_id);
    }
}
