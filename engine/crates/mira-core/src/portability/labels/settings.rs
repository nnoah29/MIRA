//! Réglages d'un libellé local sous forme portable : quatre champs facultatifs, lecture stricte.

use mira_vocabulary::{CounterMode, LabelSettings};
use serde_json::{Map, Value};

const KEYS: [&str; 4] = ["hidden", "hide_from_inbox", "notifications", "counter"];

/// Réglages lus dans le fichier ; un champ absent n'est pas touché à l'import
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PortableSettings {
    pub hidden: Option<bool>,
    pub hide_from_inbox: Option<bool>,
    pub notifications: Option<bool>,
    pub counter: Option<CounterMode>,
}

impl PortableSettings {
    /// Aucun réglage dans l'entrée
    pub fn is_empty(&self) -> bool {
        *self == Self::default()
    }

    /// Réglages de base dont les champs présents sont remplacés ; position et couleur de réglage restent
    pub fn applied_to(&self, base: LabelSettings) -> LabelSettings {
        LabelSettings {
            hidden: self.hidden.unwrap_or(base.hidden),
            hide_from_inbox: self.hide_from_inbox.unwrap_or(base.hide_from_inbox),
            notifications: self.notifications.unwrap_or(base.notifications),
            counter: self.counter.unwrap_or(base.counter),
            ..base
        }
    }
}

/// Ajoute les quatre réglages à l'objet d'un libellé
pub(super) fn write_settings(map: &mut Map<String, Value>, settings: &LabelSettings) {
    map.insert("hidden".into(), Value::Bool(settings.hidden));
    map.insert(
        "hide_from_inbox".into(),
        Value::Bool(settings.hide_from_inbox),
    );
    map.insert("notifications".into(), Value::Bool(settings.notifications));
    map.insert("counter".into(), Value::from(settings.counter.as_str()));
}

/// Compteur lu en texte strict ; toute autre valeur est refusée
fn read_counter(value: &Value) -> Option<CounterMode> {
    match value.as_str()? {
        "unread" => Some(CounterMode::Unread),
        "total" => Some(CounterMode::Total),
        "none" => Some(CounterMode::None),
        _ => None,
    }
}

/// Retire de l'objet les champs de réglage et les lit ; une valeur de mauvais type donne `None`
pub(super) fn take_settings(map: &mut Map<String, Value>) -> Option<PortableSettings> {
    let flag = |map: &mut Map<String, Value>, key: &str| match map.remove(key) {
        None => Some(None),
        Some(Value::Bool(value)) => Some(Some(value)),
        Some(_) => None,
    };
    let settings = PortableSettings {
        hidden: flag(map, KEYS[0])?,
        hide_from_inbox: flag(map, KEYS[1])?,
        notifications: flag(map, KEYS[2])?,
        counter: match map.remove(KEYS[3]) {
            None => None,
            Some(value) => Some(read_counter(&value)?),
        },
    };
    Some(settings)
}
