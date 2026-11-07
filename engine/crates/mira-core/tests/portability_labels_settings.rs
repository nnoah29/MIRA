#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Réglages d'un libellé local sous forme portable : écriture complète, lecture facultative et stricte.

use mira_core::portability::labels::{
    PortableSettings, read_portable_entry, to_portable_with_settings,
};
use mira_vocabulary::{CounterMode, LabelSettings, LocalLabel, LocalLabelId};
use serde_json::json;

/// Libellé local d'exemple
fn label() -> LocalLabel {
    LocalLabel {
        id: LocalLabelId::from_raw("l1"),
        name: "Compta".to_string(),
        color: 0xFF1E_90FF,
    }
}

/// Les quatre réglages sortent sous leurs noms de vocabulaire
#[test]
fn settings_are_written_under_vocabulary_names() {
    let settings = LabelSettings {
        hidden: true,
        hide_from_inbox: true,
        notifications: false,
        counter: CounterMode::Total,
        color: None,
        position: Some(3),
    };
    let value = to_portable_with_settings(&label(), &settings);
    assert_eq!(value["hidden"], json!(true));
    assert_eq!(value["hide_from_inbox"], json!(true));
    assert_eq!(value["notifications"], json!(false));
    assert_eq!(value["counter"], json!("total"));
    assert!(value.get("position").is_none(), "{value}");
}

/// Écrire puis relire rend les mêmes réglages
#[test]
fn settings_round_trip() {
    let settings = LabelSettings {
        hidden: true,
        hide_from_inbox: false,
        notifications: false,
        counter: CounterMode::None,
        color: None,
        position: None,
    };
    let (read, got) = read_portable_entry(to_portable_with_settings(&label(), &settings)).unwrap();
    assert_eq!(read.name, "Compta");
    assert_eq!(got.applied_to(LabelSettings::default()), settings);
}

/// Sans réglage dans l'entrée, rien n'est à appliquer
#[test]
fn entry_without_settings_has_none() {
    let (_, got) = read_portable_entry(json!({"name": "A"})).unwrap();
    assert_eq!(got, PortableSettings::default());
    assert!(got.is_empty());
}

/// Un réglage présent seul ne change que lui
#[test]
fn partial_settings_change_only_what_is_given() {
    let (_, got) = read_portable_entry(json!({"name": "A", "notifications": false})).unwrap();
    let applied = got.applied_to(LabelSettings::default());
    assert!(!applied.notifications);
    assert_eq!(applied.counter, CounterMode::Unread);
}
