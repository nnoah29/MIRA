//! Attentes bornées dans le temps pour les tests asynchrones.

use std::time::Duration;

use mira_core::events::RecordingSink;
use mira_vocabulary::EventKind;

/// Attend que la condition devienne vraie (vérifiée toutes les 20 ms), au plus 15 secondes
pub async fn wait_until(mut condition: impl FnMut() -> bool) {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(15);
    while !condition() {
        assert!(
            tokio::time::Instant::now() < deadline,
            "condition jamais remplie"
        );
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
}

/// Nombre d'événements émis qui satisfont le prédicat
pub fn count_events(sink: &RecordingSink, predicate: impl Fn(&EventKind) -> bool) -> usize {
    sink.kinds().iter().filter(|kind| predicate(kind)).count()
}

/// Nombre d'événements « retour en ligne » émis
pub fn back_online(sink: &RecordingSink) -> usize {
    count_events(sink, |k| matches!(k, EventKind::BackOnline { .. }))
}

/// Nombre d'événements « passage hors ligne » émis
pub fn went_offline(sink: &RecordingSink) -> usize {
    count_events(sink, |k| matches!(k, EventKind::WentOffline { .. }))
}

/// Nombre d'événements de diagnostic pour cette cible
pub fn diagnostics_for(sink: &RecordingSink, wanted: &str) -> usize {
    count_events(
        sink,
        |k| matches!(k, EventKind::Diagnostic { target, .. } if target == wanted),
    )
}

/// Nombre d'événements `SyncFinished` émis jusqu'ici
pub fn finished_passes(sink: &RecordingSink) -> usize {
    count_events(sink, |kind| matches!(kind, EventKind::SyncFinished { .. }))
}
