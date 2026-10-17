//! Pauses des tests qui vérifient qu'une tâche de fond n'a rien fait.

use std::time::Duration;

/// Laisse aux tâches de fond le temps d'agir, pour constater qu'elles n'ont rien fait
pub async fn settle() {
    tokio::time::sleep(Duration::from_millis(700)).await;
}
