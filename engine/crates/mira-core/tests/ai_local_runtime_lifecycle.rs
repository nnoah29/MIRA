#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Cycle de vie du serveur local : inactivité, changement de modèle, arrêt sans orphelin, mémoire juste.

use std::time::Duration;

use mira_core::ai::local::LocalRuntimeManager;
use mira_core::ports::FixedResources;

mod local_support;
use local_support::runtime::RuntimeFixture;
use local_support::test_model_named;

/// Sans sollicitation pendant le délai d'inactivité, le serveur est arrêté
#[tokio::test]
async fn idle_timeout_stops_the_server() {
    let mut fixture = RuntimeFixture::new(&[]);
    fixture.runtime.idle_stop_seconds = 1;
    let manager = LocalRuntimeManager::new();
    let model = test_model_named(&fixture, "mini");
    manager
        .ensure_running(&fixture.context(), &model)
        .await
        .unwrap();
    let pid = fixture.starts()[0]["pid"].as_u64().unwrap();
    tokio::time::sleep(Duration::from_millis(2_500)).await;
    assert_eq!(manager.running_model().await, None);
    assert!(!RuntimeFixture::alive(pid));
}

/// Une sollicitation avant le délai repousse l'arrêt
#[tokio::test]
async fn use_before_the_delay_keeps_the_server_alive() {
    let mut fixture = RuntimeFixture::new(&[]);
    fixture.runtime.idle_stop_seconds = 2;
    let manager = LocalRuntimeManager::new();
    let model = test_model_named(&fixture, "mini");
    for _ in 0..3 {
        manager
            .ensure_running(&fixture.context(), &model)
            .await
            .unwrap();
        tokio::time::sleep(Duration::from_millis(1_000)).await;
    }
    assert_eq!(manager.running_model().await.as_deref(), Some("mini"));
    assert_eq!(fixture.starts().len(), 1);
    manager.stop().await;
}

/// Changer de modèle arrête l'ancien serveur et en lance un nouveau sous le nouvel alias
#[tokio::test]
async fn switching_model_restarts_with_the_new_alias() {
    let fixture = RuntimeFixture::new(&[]);
    let manager = LocalRuntimeManager::new();
    let (mini, big) = (
        test_model_named(&fixture, "mini"),
        test_model_named(&fixture, "big"),
    );
    manager
        .ensure_running(&fixture.context(), &mini)
        .await
        .unwrap();
    let old_pid = fixture.starts()[0]["pid"].as_u64().unwrap();
    manager
        .ensure_running(&fixture.context(), &big)
        .await
        .unwrap();
    let starts = fixture.starts();
    assert_eq!(starts[1]["alias"], "big");
    assert!(!RuntimeFixture::alive(old_pid));
    manager.stop().await;
}

/// L'arrêt demandé ne laisse aucun processus
#[tokio::test]
async fn stop_leaves_no_orphan_process() {
    let fixture = RuntimeFixture::new(&[]);
    let manager = LocalRuntimeManager::new();
    let model = test_model_named(&fixture, "mini");
    manager
        .ensure_running(&fixture.context(), &model)
        .await
        .unwrap();
    let pid = fixture.starts()[0]["pid"].as_u64().unwrap();
    manager.stop().await;
    assert!(!RuntimeFixture::alive(pid));
    assert_eq!(manager.running_model().await, None);
}

/// Un modèle absent du disque donne l'erreur « modèle manquant » sans lancer de serveur
#[tokio::test]
async fn missing_model_file_is_a_clear_error_without_launch() {
    let fixture = RuntimeFixture::new(&[]);
    std::fs::remove_file(fixture.dir.path().join("models").join("mini.bin")).unwrap();
    let manager = LocalRuntimeManager::new();
    let model = test_model_named(&fixture, "mini");
    let error = manager
        .ensure_running(&fixture.context(), &model)
        .await
        .unwrap_err();
    assert_eq!(error.code, "incompatible_state.ai_model_missing");
    assert!(fixture.starts().is_empty());
}

/// Une sortie du serveur avec peu de mémoire disponible le dit dans le message
#[tokio::test]
async fn low_memory_adds_a_distinct_message_to_the_failure() {
    let mut fixture = RuntimeFixture::new(&["--fake-exit-code", "1"]);
    fixture.resources = FixedResources::new(Some(100), Some(u64::MAX));
    let manager = LocalRuntimeManager::new();
    let model = test_model_named(&fixture, "mini");
    let error = manager
        .ensure_running(&fixture.context(), &model)
        .await
        .unwrap_err();
    assert_eq!(error.code, "server.ai.local_runtime_failed");
    assert!(error.message.contains("mémoire"), "{}", error.message);
}
