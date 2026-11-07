//! L'icône MIRA vue par le bureau : identité, clic gauche, menu.

use ksni::{Category, MenuItem};

use super::menu::entries;
use super::status::TrayStatus;
use crate::desktop::Launch;

const APP_ID: &str = "io.github.nnoah29.Mira";

/// Icône et ses actions : relancer l'app, quitter le démon.
pub(super) struct MiraTray {
    pub status: TrayStatus,
    pub locale: String,
    pub launch: Launch,
    pub quit: Launch,
}

impl ksni::Tray for MiraTray {
    /// Identité stable de l'icône.
    fn id(&self) -> String {
        APP_ID.to_string()
    }

    /// Nom affiché.
    fn title(&self) -> String {
        "MIRA".to_string()
    }

    /// Icône du thème.
    fn icon_name(&self) -> String {
        APP_ID.to_string()
    }

    /// Application de messagerie.
    fn category(&self) -> Category {
        Category::Communications
    }

    /// Clic gauche : relance l'app.
    fn activate(&mut self, _x: i32, _y: i32) {
        (self.launch)();
    }

    /// Ouvrir, état, quitter.
    fn menu(&self) -> Vec<MenuItem<Self>> {
        entries(self)
    }
}
