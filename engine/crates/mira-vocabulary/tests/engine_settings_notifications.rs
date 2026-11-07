#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Réglages de notification du moteur : défauts d'avant la phase 4, lecture tolérante, clé annoncée.

use mira_vocabulary::{EngineSettings, NotificationSettings};

/// Réglages dont seul le bloc de notifications est donné.
fn with(notifications: NotificationSettings) -> EngineSettings {
    EngineSettings {
        notifications,
        ..EngineSettings::default()
    }
}

/// Les défauts sont ceux de l'app avant la phase 4 : activé, aperçu, son, pas au premier plan.
#[test]
fn defaults_match_the_previous_app_defaults() {
    let expected = NotificationSettings {
        enabled: true,
        preview: true,
        sound: true,
        when_focused: false,
    };
    assert_eq!(NotificationSettings::default(), expected);
}

/// Un JSON de moteur 1.6, sans bloc de notifications, se lit avec les défauts.
#[test]
fn json_from_a_1_6_engine_loads_the_defaults() {
    let old = r#"{"poll_secs":60,"muted_accounts":[],"quiet_hours":null}"#;
    let settings: EngineSettings = serde_json::from_str(old).unwrap();
    assert_eq!(settings.notifications, NotificationSettings::default());
}

/// Un bloc partiel garde les défauts pour les champs absents.
#[test]
fn a_partial_block_keeps_the_other_defaults() {
    let settings: EngineSettings =
        serde_json::from_str(r#"{"notifications":{"sound":false}}"#).unwrap();
    let expected = NotificationSettings {
        sound: false,
        ..NotificationSettings::default()
    };
    assert_eq!(settings.notifications, expected);
}

/// Un changement du bloc est annoncé par la clé `notifications`, rien d'autre.
#[test]
fn a_change_is_reported_by_the_notifications_key() {
    let quiet = NotificationSettings {
        enabled: false,
        ..NotificationSettings::default()
    };
    let keys = EngineSettings::default().changed_keys(&with(quiet));
    assert_eq!(keys, ["notifications"]);
}

/// Les noms JSON sont en minuscules avec soulignés, comme le reste du contrat.
#[test]
fn field_names_are_snake_case() {
    let json = serde_json::to_value(NotificationSettings::default()).unwrap();
    let mut names: Vec<_> = json.as_object().unwrap().keys().cloned().collect();
    names.sort();
    assert_eq!(names, ["enabled", "preview", "sound", "when_focused"]);
}
