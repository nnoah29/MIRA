//! Lectures des événements émis pendant une synchronisation.

use mira_core::events::RecordingSink;
use mira_vocabulary::{EventKind, SyncStep};

use super::waiting::count_events;

/// Couples (fait, total) des événements de progression de l'étape des en-têtes, dans l'ordre
pub fn header_progress(sink: &RecordingSink) -> Vec<(u64, u64)> {
    sink.kinds()
        .iter()
        .filter_map(|kind| match kind {
            EventKind::SyncProgress {
                step: SyncStep::Headers,
                done,
                total,
                ..
            } => Some((*done, *total)),
            _ => None,
        })
        .collect()
}

/// Y a-t-il au moins un événement de progression pour cette étape ?
pub fn has_progress_step(sink: &RecordingSink, wanted: SyncStep) -> bool {
    let is_wanted =
        |kind: &EventKind| matches!(kind, EventKind::SyncProgress { step, .. } if *step == wanted);
    count_events(sink, is_wanted) > 0
}

/// Nombre d'événements « nouveau message »
pub fn announced(sink: &RecordingSink) -> usize {
    count_events(sink, |k| matches!(k, EventKind::NewMessageReceived { .. }))
}

/// Nombre d'événements « message supprimé »
pub fn deleted(sink: &RecordingSink) -> usize {
    count_events(sink, |k| matches!(k, EventKind::MessageDeleted { .. }))
}

/// Nombre d'événements « boîte resynchronisée »
pub fn resynchronized(sink: &RecordingSink) -> usize {
    count_events(sink, |k| {
        matches!(k, EventKind::MailboxResynchronized { .. })
    })
}
