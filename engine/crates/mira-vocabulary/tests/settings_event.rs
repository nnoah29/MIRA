#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use mira_vocabulary::{EventCategory, EventKind};

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
