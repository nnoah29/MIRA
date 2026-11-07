//! Flux d'événements : le journal passé n'est rejoué que sur demande explicite.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use super::driver::Driver;

/// Compte les événements reçus pendant 600 ms à partir de ce numéro.
async fn collect<D: Driver>(driver: &D, after: Option<u64>) -> usize {
    let seen = Arc::new(AtomicUsize::new(0));
    let counter = Arc::clone(&seen);
    let pump = driver.pump_events(after, move |_| {
        counter.fetch_add(1, Ordering::Relaxed);
        true
    });
    let _ = tokio::time::timeout(Duration::from_millis(600), pump).await;
    seen.load(Ordering::Relaxed)
}

/// Sans numéro de départ, le journal passé n'est pas rejoué ; avec 0, il l'est.
pub async fn check_no_event_replay<D: Driver>(driver: &D) {
    assert_eq!(
        collect(driver, None).await,
        0,
        "le journal passé ne doit pas être rejoué"
    );
    assert!(
        collect(driver, Some(0)).await > 0,
        "depuis 0, le journal est rejoué"
    );
}
