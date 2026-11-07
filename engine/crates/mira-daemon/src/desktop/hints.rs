//! Détails d'affichage d'une notification : catégorie, son, libellé de l'action.

use std::collections::HashMap;

use zbus::zvariant::Value;

/// Indications pour le service : catégorie « courrier arrivé », et le son du système ou son absence.
pub fn hints_for(sound: bool) -> HashMap<&'static str, Value<'static>> {
    let mut hints = HashMap::new();
    hints.insert("category", Value::from("email.arrived"));
    if sound {
        hints.insert("sound-name", Value::from("message-new-email"));
    } else {
        hints.insert("suppress-sound", Value::from(true));
    }
    hints
}

/// Libellé de l'action par défaut selon la langue du système.
pub fn action_label(locale: &str) -> &'static str {
    if locale.starts_with("fr") {
        "Ouvrir"
    } else {
        "Open"
    }
}
