#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Réglages de notification par la façade : gardés, annoncés, rétablis aux défauts.

mod support_api;
mod support_settings;

use mira_vocabulary::{EngineSettings, NotificationSettings};
use support_api::fixture;
use support_settings::calls::{announced_keys, current, replace};

/// Réglages tous contraires aux défauts.
fn opposite() -> EngineSettings {
    EngineSettings {
        notifications: NotificationSettings {
            enabled: false,
            preview: false,
            sound: false,
            when_focused: true,
        },
        ..EngineSettings::default()
    }
}

/// Un profil neuf a les réglages de notification par défaut.
#[tokio::test]
async fn a_fresh_profile_has_the_default_notification_settings() {
    let f = fixture::start().await;
    assert_eq!(
        current(&f.engine).await.notifications,
        NotificationSettings::default()
    );
}

/// Des réglages contraires aux défauts sont gardés et annoncés par la clé `notifications`.
#[tokio::test]
async fn notification_settings_are_stored_and_announced() {
    let f = fixture::start().await;
    replace(&f.engine, opposite()).await.unwrap();
    assert_eq!(current(&f.engine).await, opposite());
    assert_eq!(announced_keys(&f.engine).await, [["notifications"]]);
}

/// Revenir aux défauts les rétablit et l'annonce de nouveau.
#[tokio::test]
async fn restoring_the_defaults_is_stored_and_announced() {
    let f = fixture::start().await;
    replace(&f.engine, opposite()).await.unwrap();
    replace(&f.engine, EngineSettings::default()).await.unwrap();
    assert_eq!(current(&f.engine).await, EngineSettings::default());
    let keys = announced_keys(&f.engine).await;
    assert_eq!(keys, [["notifications"], ["notifications"]]);
}
