#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use mira_vocabulary::{EngineSettings, EventCategory, EventKind};

/// Deux réglages identiques ne diffèrent sur aucune clé
#[test]
fn identical_settings_have_no_changed_key() {
    let settings = EngineSettings {
        poll_secs: Some(60),
    };
    assert!(settings.changed_keys(&settings).is_empty());
}

/// Un relevé modifié, ajouté ou retiré est signalé par sa clé
#[test]
fn changed_poll_is_reported_by_its_key() {
    let some = |secs| EngineSettings {
        poll_secs: Some(secs),
    };
    let none = EngineSettings::default();
    assert_eq!(some(60).changed_keys(&some(90)), ["poll_secs"]);
    assert_eq!(none.changed_keys(&some(90)), ["poll_secs"]);
    assert_eq!(some(60).changed_keys(&none), ["poll_secs"]);
}

/// L'annonce de réglages changés est de catégorie « settings », sans compte, et se sérialise avec ses clés
#[test]
fn settings_changed_event_has_its_own_category_and_no_account() {
    let event = EventKind::SettingsChanged {
        keys: vec!["poll_secs".to_string()],
    };
    assert_eq!(event.category(), EventCategory::Settings);
    assert!(event.account().is_none());
    let json = serde_json::to_value(&event).unwrap();
    assert_eq!(json["type"], "settings_changed");
    assert_eq!(json["keys"][0], "poll_secs");
}
