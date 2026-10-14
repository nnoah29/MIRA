//! Nonce de session : tiré de l'horloge, jamais le même d'une session à l'autre.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::HashSet;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use mira_tui::app::session_nonce::{session_nonce, session_nonce_from};

/// Deux sessions ouvertes l'une après l'autre, même dans la même milliseconde, ont des nonces différents.
#[test]
fn two_sessions_get_different_nonces() {
    let nonces: HashSet<u64> = (0..1_000).map(|_| session_nonce()).collect();
    assert_eq!(nonces.len(), 1_000);
}

/// Le nonce suit l'horloge injectée ; une horloge avant 1970 ne fait pas paniquer.
#[test]
fn nonce_follows_the_injected_clock() {
    let early = UNIX_EPOCH + Duration::from_millis(1_700_000_000_000);
    let later = early + Duration::from_millis(1);
    assert_ne!(
        session_nonce_from(early, 42, 0),
        session_nonce_from(later, 42, 0)
    );
    assert_ne!(
        session_nonce_from(early, 42, 0),
        session_nonce_from(early, 43, 0)
    );
    assert_ne!(
        session_nonce_from(early, 42, 0),
        session_nonce_from(early, 42, 1)
    );
    let before_epoch = UNIX_EPOCH - Duration::from_secs(10);
    let _ = session_nonce_from(before_epoch, 1, 0);
    assert_ne!(session_nonce_from(SystemTime::now(), 1, 0), 0);
}
