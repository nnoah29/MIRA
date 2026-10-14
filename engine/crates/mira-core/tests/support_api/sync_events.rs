//! Lectures de la suite d'événements d'une synchronisation : progression des en-têtes, décompte par nature.

use mira_vocabulary::{EventKind, SyncStep};

/// Couples (fait, total) des événements de progression de l'étape des en-têtes, dans l'ordre
pub fn header_progress(kinds: &[EventKind]) -> Vec<(u64, u64)> {
    let headers = |kind: &EventKind| match kind {
        EventKind::SyncProgress {
            step: SyncStep::Headers,
            done,
            total,
            ..
        } => Some((*done, *total)),
        _ => None,
    };
    kinds.iter().filter_map(headers).collect()
}

/// Nombre d'événements qui satisfont le prédicat
pub fn count_kinds(kinds: &[EventKind], predicate: impl Fn(&EventKind) -> bool) -> usize {
    kinds.iter().filter(|kind| predicate(kind)).count()
}
