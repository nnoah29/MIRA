#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Appels bloquants du trousseau : ils ne gèlent pas l'exécuteur asynchrone (audit 7).

use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;

use mira_core::ports::run_blocking_call;

/// Pendant un appel bloquant de 400 ms, une autre tâche continue de progresser sur un seul fil
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn regression_blocking_call_does_not_starve_other_tasks() {
    let ticks = Arc::new(AtomicU32::new(0));
    let counter = Arc::clone(&ticks);
    let ticker = tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_millis(10)).await;
            counter.fetch_add(1, Ordering::SeqCst);
        }
    });
    let blocked = tokio::spawn(async {
        run_blocking_call(|| std::thread::sleep(Duration::from_millis(400)));
    });
    tokio::time::timeout(Duration::from_secs(10), blocked)
        .await
        .unwrap()
        .unwrap();
    ticker.abort();
    assert!(ticks.load(Ordering::SeqCst) >= 10);
}

/// Sur un exécuteur à un seul fil, l'appel s'exécute directement et rend sa valeur
#[tokio::test]
async fn blocking_call_on_current_thread_runtime_returns_value() {
    assert_eq!(run_blocking_call(|| 7), 7);
}
