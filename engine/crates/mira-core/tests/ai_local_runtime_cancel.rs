#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Lancement lent : l'état ne l'attend pas, l'arrêt demandé l'interrompt sans laisser de processus.

use std::time::Duration;

use mira_core::ai::local::LocalRuntimeManager;

mod local_support;
use local_support::runtime::RuntimeFixture;
use local_support::test_model_named;

/// Pendant un chargement lent, l'état répond tout de suite au lieu d'attendre la fin du lancement
#[tokio::test]
async fn status_does_not_wait_for_a_slow_launch() {
    let fixture = RuntimeFixture::new(&["--fake-ready-after-ms", "1500"]);
    let manager = LocalRuntimeManager::new();
    let model = test_model_named(&fixture, "mini");
    let ctx = fixture.context();
    let launch = manager.ensure_running(&ctx, &model);
    let probe = async {
        tokio::time::sleep(Duration::from_millis(300)).await;
        let began = std::time::Instant::now();
        let running = manager.running_model().await;
        (began.elapsed(), running)
    };
    let (launched, (waited, running)) = tokio::join!(launch, probe);
    launched.unwrap();
    assert!(waited < Duration::from_millis(200), "attente de {waited:?}");
    assert_eq!(running, None);
    manager.stop().await;
}

/// L'arrêt demandé pendant un chargement lent l'interrompt et ne laisse aucun processus
#[tokio::test]
async fn stop_during_a_slow_launch_cancels_it_without_orphan() {
    let fixture = RuntimeFixture::new(&["--fake-hang"]);
    let manager = LocalRuntimeManager::new();
    let model = test_model_named(&fixture, "mini");
    let ctx = fixture.context();
    let launch = manager.ensure_running(&ctx, &model);
    let stopper = async {
        tokio::time::sleep(Duration::from_millis(500)).await;
        let began = std::time::Instant::now();
        manager.stop().await;
        began.elapsed()
    };
    let (launched, waited) = tokio::join!(launch, stopper);
    assert!(launched.is_err());
    assert!(waited < Duration::from_secs(2), "arrêt attendu {waited:?}");
    let pid = fixture.starts()[0]["pid"].as_u64().unwrap();
    tokio::time::sleep(Duration::from_millis(300)).await;
    assert!(!RuntimeFixture::alive(pid));
}
