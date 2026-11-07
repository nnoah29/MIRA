//! Outils des tests D-Bus : notification d'exemple, compteur de relances de l'app, attente d'une condition.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use mira_host::notifier::Notification;

/// Notification d'exemple.
pub fn sample(sound: bool) -> Notification {
    Notification {
        id: 7,
        title: "Bob".into(),
        body: "Rendez-vous demain".into(),
        sound,
        message: Some("a1/m1".into()),
    }
}

/// Compteur de lancements de l'app et fonction qui l'incrémente.
pub fn counting() -> (Arc<AtomicUsize>, mira_daemon::desktop::Launch) {
    let count = Arc::new(AtomicUsize::new(0));
    let seen = count.clone();
    let launch: mira_daemon::desktop::Launch = Arc::new(move || {
        seen.fetch_add(1, Ordering::SeqCst);
    });
    (count, launch)
}

/// Attend, au plus cinq secondes, que la condition devienne vraie.
pub async fn until(mut condition: impl FnMut() -> bool) -> bool {
    for _ in 0..100 {
        if condition() {
            return true;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    false
}
