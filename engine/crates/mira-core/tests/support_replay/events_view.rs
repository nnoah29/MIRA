//! Lecture des événements d'action et d'envoi émis pendant un test.

use mira_vocabulary::EventKind;

use super::fixture::Fixture;

/// Codes des événements `ActionFailed` émis jusqu'ici
pub fn failure_codes(fx: &Fixture) -> Vec<String> {
    let failed = |kind: EventKind| match kind {
        EventKind::ActionFailed { code, .. } => Some(code),
        _ => None,
    };
    fx.world
        .sink
        .kinds()
        .into_iter()
        .filter_map(failed)
        .collect()
}

/// Genres d'événements d'envoi émis jusqu'ici, sans leurs détails
pub fn send_events(fx: &Fixture) -> Vec<&'static str> {
    let named = |kind: EventKind| match kind {
        EventKind::SendScheduled { .. } => Some("scheduled"),
        EventKind::SendDeparted { .. } => Some("departed"),
        EventKind::SentCopyStored { .. } => Some("sent_copy"),
        EventKind::SendFailed { .. } => Some("failed"),
        EventKind::SendUncertain { .. } => Some("uncertain"),
        _ => None,
    };
    fx.world
        .sink
        .kinds()
        .into_iter()
        .filter_map(named)
        .collect()
}

/// Code du premier événement `SendFailed`, s'il y en a un
pub fn send_failure_code(fx: &Fixture) -> Option<String> {
    fx.world
        .sink
        .kinds()
        .into_iter()
        .find_map(|kind| match kind {
            EventKind::SendFailed { code, .. } => Some(code),
            _ => None,
        })
}
