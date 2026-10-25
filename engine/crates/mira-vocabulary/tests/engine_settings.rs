#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use mira_vocabulary::{AccountId, EngineSettings, QuietHours};

/// Réglages avec un relevé donné
fn polling(secs: u32) -> EngineSettings {
    EngineSettings {
        poll_secs: Some(secs),
        ..EngineSettings::default()
    }
}

/// Deux réglages identiques ne diffèrent sur aucune clé
#[test]
fn identical_settings_have_no_changed_key() {
    assert!(polling(60).changed_keys(&polling(60)).is_empty());
}

/// Un relevé modifié, ajouté ou retiré est signalé par sa clé
#[test]
fn changed_poll_is_reported_by_its_key() {
    let none = EngineSettings::default();
    assert_eq!(polling(60).changed_keys(&polling(90)), ["poll_secs"]);
    assert_eq!(none.changed_keys(&polling(90)), ["poll_secs"]);
    assert_eq!(polling(60).changed_keys(&none), ["poll_secs"]);
}

/// Sourdine et heures calmes sont signalées chacune par leur clé, dans un ordre stable
#[test]
fn muted_accounts_and_quiet_hours_are_reported_in_stable_order() {
    let changed = EngineSettings {
        poll_secs: Some(60),
        muted_accounts: vec![AccountId::from_raw("a1")],
        quiet_hours: Some(QuietHours { from: 22, to: 7 }),
    };
    let keys = polling(60).changed_keys(&changed);
    assert_eq!(keys, ["muted_accounts", "quiet_hours"]);
}

/// Un compte listé est en sourdine, un autre non
#[test]
fn only_listed_accounts_are_muted() {
    let settings = EngineSettings {
        muted_accounts: vec![AccountId::from_raw("a1")],
        ..EngineSettings::default()
    };
    assert!(settings.mutes(&AccountId::from_raw("a1")));
    assert!(!settings.mutes(&AccountId::from_raw("a2")));
}

/// Un ancien client qui n'envoie que le relevé obtient les valeurs par défaut des autres réglages
#[test]
fn settings_missing_fields_take_their_defaults() {
    let parsed: EngineSettings = serde_json::from_str(r#"{"poll_secs":120}"#).unwrap();
    assert_eq!(parsed, polling(120));
}
