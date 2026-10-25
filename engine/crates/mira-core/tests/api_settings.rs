#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Domaine « settings » par la façade : lecture, remplacement, annonce unique, persistance, application immédiate.

mod support_api;
mod support_settings;

use mira_vocabulary::EngineSettings;
use support_api::fixture;
use support_api::sync_calls::{finished_passes, wait_for_events};
use support_api::synced::synced_account;
use support_settings::calls::{announced_keys, current, set_poll};

/// Sans réglage enregistré, le relevé reste au choix du moteur
#[tokio::test]
async fn fresh_profile_has_default_settings() {
    let f = fixture::start().await;
    assert_eq!(current(&f.engine).await, EngineSettings::default());
    assert!(announced_keys(&f.engine).await.is_empty());
}

/// Un changement est enregistré et annoncé une fois ; le même réglage redemandé n'annonce rien
#[tokio::test]
async fn set_stores_and_announces_only_real_changes() {
    let f = fixture::start().await;
    let stored = set_poll(&f.engine, Some(120)).await.unwrap();
    assert_eq!(stored.poll_secs, Some(120));
    set_poll(&f.engine, Some(120)).await.unwrap();
    assert_eq!(current(&f.engine).await.poll_secs, Some(120));
    assert_eq!(announced_keys(&f.engine).await, [["poll_secs".to_string()]]);
    set_poll(&f.engine, None).await.unwrap();
    assert_eq!(announced_keys(&f.engine).await.len(), 2);
}

/// Un relevé hors des bornes est refusé et rien n'est modifié ni annoncé
#[tokio::test]
async fn out_of_range_poll_is_refused_without_side_effect() {
    let f = fixture::start().await;
    for secs in [0, 3601] {
        let code = set_poll(&f.engine, Some(secs)).await.unwrap_err();
        assert_eq!(code, "invalid_input.parameter");
    }
    assert_eq!(current(&f.engine).await, EngineSettings::default());
    assert!(announced_keys(&f.engine).await.is_empty());
}

/// Les réglages survivent à un redémarrage du moteur
#[tokio::test]
async fn settings_survive_a_restart() {
    let f = fixture::start().await;
    set_poll(&f.engine, Some(90)).await.unwrap();
    let reopened = f.reopen();
    assert_eq!(current(&reopened.engine).await.poll_secs, Some(90));
}

/// Un nouveau relevé s'applique sans redémarrage : avec 600 s au départ, la passe suivante suit le réglage à 1 s
#[tokio::test]
async fn changed_poll_applies_to_running_accounts_without_restart() {
    let (f, _account) = synced_account().await;
    let before = finished_passes(&wait_for_events(&f.engine, |_| true).await);
    set_poll(&f.engine, Some(1)).await.unwrap();
    wait_for_events(&f.engine, |kinds| finished_passes(kinds) > before).await;
    assert_eq!(announced_keys(&f.engine).await.len(), 1);
}
