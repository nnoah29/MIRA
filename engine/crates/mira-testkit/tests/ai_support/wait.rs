//! Attente d'une condition observable sur le faux serveur, avec une borne de temps.

use std::time::Duration;

use tokio::time::{sleep, timeout};

use super::LIMIT;

/// Attend que la condition devienne vraie, en la sondant toutes les 10 ms
pub async fn until(condition: impl Fn() -> bool) {
    timeout(LIMIT, async {
        while !condition() {
            sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap();
}
