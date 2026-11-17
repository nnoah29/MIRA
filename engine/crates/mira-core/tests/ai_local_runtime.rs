#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Gestionnaire du serveur local : lancement unique, adresse et clé du lancement, arrêt propre.

use std::os::unix::fs::PermissionsExt;

use mira_core::ai::local::LocalRuntimeManager;

mod local_support;
use local_support::runtime::RuntimeFixture;
use local_support::test_model_named;

/// Le premier appel lance le serveur, attend l'état prêt et rend l'adresse de boucle locale avec la clé
#[tokio::test]
async fn starts_waits_for_health_and_returns_loopback_url_and_key() {
    let fixture = RuntimeFixture::new(&["--fake-ready-after-ms", "600"]);
    let manager = LocalRuntimeManager::new();
    let model = test_model_named(&fixture, "mini");
    let handle = manager
        .ensure_running(&fixture.context(), &model)
        .await
        .unwrap();
    assert!(handle.base_url.starts_with("http://127.0.0.1:") && handle.base_url.ends_with("/v1"));
    let health = reqwest::get(format!("{}/health", handle.root_url))
        .await
        .unwrap();
    assert_eq!(health.status(), 200);
    assert_eq!(fixture.starts().len(), 1);
    manager.stop().await;
}

/// Deux premiers appels simultanés ne lancent qu'un serveur
#[tokio::test]
async fn concurrent_first_requests_start_one_server() {
    let fixture = RuntimeFixture::new(&[]);
    let manager = LocalRuntimeManager::new();
    let model = test_model_named(&fixture, "mini");
    let ctx = fixture.context();
    let (a, b) = tokio::join!(
        manager.ensure_running(&ctx, &model),
        manager.ensure_running(&ctx, &model)
    );
    assert_eq!(a.unwrap().base_url, b.unwrap().base_url);
    assert_eq!(fixture.starts().len(), 1);
    manager.stop().await;
}

/// La clé n'est jamais dans la ligne de commande, change à chaque lancement, fichier en 0600
#[tokio::test]
async fn key_is_not_in_the_command_line_and_differs_between_launches() {
    let fixture = RuntimeFixture::new(&[]);
    let manager = LocalRuntimeManager::new();
    let model = test_model_named(&fixture, "mini");
    let first = manager
        .ensure_running(&fixture.context(), &model)
        .await
        .unwrap();
    manager.stop().await;
    let second = manager
        .ensure_running(&fixture.context(), &model)
        .await
        .unwrap();
    manager.stop().await;
    let starts = fixture.starts();
    assert_eq!(starts.len(), 2);
    assert_ne!(starts[0]["key"], starts[1]["key"]);
    for start in &starts {
        let key = start["key"].as_str().unwrap();
        assert_eq!(key.len(), 64);
        assert!(!start["argv"].to_string().contains(key));
    }
    assert!(first.base_url != second.base_url || starts[0]["key"] != starts[1]["key"]);
    let mode = std::fs::metadata(fixture.run_dir().join("api-key"))
        .unwrap()
        .permissions()
        .mode();
    assert_eq!(mode & 0o777, 0o600);
}

/// Un serveur jamais prêt dans le délai est tué et l'erreur de délai est rendue
#[tokio::test]
async fn start_timeout_returns_start_timeout_error_and_kills_the_child() {
    let mut fixture = RuntimeFixture::new(&["--fake-hang"]);
    fixture.runtime.start_timeout_seconds = 1;
    let manager = LocalRuntimeManager::new();
    let model = test_model_named(&fixture, "mini");
    let error = manager
        .ensure_running(&fixture.context(), &model)
        .await
        .unwrap_err();
    assert_eq!(error.code, "server.ai_local_start_timeout");
    let pid = fixture.starts()[0]["pid"].as_u64().unwrap();
    tokio::time::sleep(std::time::Duration::from_millis(300)).await;
    assert!(!RuntimeFixture::alive(pid));
}

/// Un serveur qui sort avant d'être prêt donne l'erreur d'échec d'exécution
#[tokio::test]
async fn child_exit_before_ready_returns_runtime_failed() {
    let fixture = RuntimeFixture::new(&["--fake-exit-code", "3"]);
    let manager = LocalRuntimeManager::new();
    let model = test_model_named(&fixture, "mini");
    let error = manager
        .ensure_running(&fixture.context(), &model)
        .await
        .unwrap_err();
    assert_eq!(error.code, "server.ai_local_runtime_failed");
}

/// Après un plantage en cours d'usage, l'appel suivant relance un seul serveur
#[tokio::test]
async fn child_crash_while_running_then_next_call_relaunches_once() {
    let fixture = RuntimeFixture::new(&[]);
    let manager = LocalRuntimeManager::new();
    let model = test_model_named(&fixture, "mini");
    let handle = manager
        .ensure_running(&fixture.context(), &model)
        .await
        .unwrap();
    let _ = reqwest::get(format!("{}/die", handle.root_url)).await;
    tokio::time::sleep(std::time::Duration::from_millis(300)).await;
    manager
        .ensure_running(&fixture.context(), &model)
        .await
        .unwrap();
    assert_eq!(fixture.starts().len(), 2);
    manager.stop().await;
}
