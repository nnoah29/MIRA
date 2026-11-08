//! Entrées du menu de l'icône : « Ouvrir MIRA », la ligne d'état, « Quitter MIRA ».

use ksni::MenuItem;
use ksni::menu::StandardItem;

use super::item::MiraTray;
use super::status::{menu_labels, status_line};

/// Les trois entrées, dans l'ordre : ouvrir, état (grisée), quitter.
pub(super) fn entries(tray: &MiraTray) -> Vec<MenuItem<MiraTray>> {
    let (open, quit) = menu_labels(&tray.locale);
    vec![
        StandardItem {
            label: open.to_string(),
            activate: Box::new(|tray: &mut MiraTray| (tray.launch)()),
            ..StandardItem::default()
        }
        .into(),
        StandardItem {
            label: status_line(tray.status, &tray.locale).to_string(),
            enabled: false,
            ..StandardItem::default()
        }
        .into(),
        StandardItem {
            label: quit.to_string(),
            activate: Box::new(|tray: &mut MiraTray| (tray.quit)()),
            ..StandardItem::default()
        }
        .into(),
    ]
}
