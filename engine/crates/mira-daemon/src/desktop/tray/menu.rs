//! Entrées du menu de l'icône : « Ouvrir MIRA », la ligne d'état, « Quitter MIRA ».

use ksni::MenuItem;
use ksni::menu::StandardItem;

use super::item::MiraTray;
use super::status::status_line;

/// Libellés « Ouvrir » et « Quitter » selon la langue.
fn labels(locale: &str) -> (&'static str, &'static str) {
    if locale.starts_with("fr") {
        ("Ouvrir MIRA", "Quitter MIRA")
    } else {
        ("Open MIRA", "Quit MIRA")
    }
}

/// Les trois entrées, dans l'ordre : ouvrir, état (grisée), quitter.
pub(super) fn entries(tray: &MiraTray) -> Vec<MenuItem<MiraTray>> {
    let (open, quit) = labels(&tray.locale);
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
