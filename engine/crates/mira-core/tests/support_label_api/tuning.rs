//! Réglages d'un libellé local posés et relus par la façade, pour les tests d'export et d'import.

use mira_core::api::{Engine, LabelSettingsRequest};
use mira_vocabulary::{CounterMode, LabelSettings};

use super::settings::{call, labels, local_ref};

/// Règle les quatre réglages d'un libellé local
pub async fn tune(engine: &Engine, id: &str, settings: &LabelSettings) {
    let request = LabelSettingsRequest::SetLabelSettings {
        label: local_ref(id),
        hidden: Some(settings.hidden),
        hide_from_inbox: Some(settings.hide_from_inbox),
        notifications: Some(settings.notifications),
        counter: Some(settings.counter),
        color: None,
    };
    call(engine, request).await.unwrap();
}

/// Réglages d'un libellé local, par son nom
pub async fn settings_of(engine: &Engine, name: &str) -> LabelSettings {
    let all = labels(engine).await;
    all.into_iter().find(|l| l.name == name).unwrap().settings
}

/// Réglages choisis à la main, tous différents des défauts
pub fn chosen() -> LabelSettings {
    LabelSettings {
        hidden: true,
        hide_from_inbox: true,
        notifications: false,
        counter: CounterMode::Total,
        color: None,
        position: None,
    }
}
