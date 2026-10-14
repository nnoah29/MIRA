//! Nonce de session : distingue les clés d'idempotence d'un lancement à l'autre.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static SESSIONS: AtomicU64 = AtomicU64::new(0);

/// Nonce de cette session : horloge système, processus et rang de la session dans le processus.
pub fn session_nonce() -> u64 {
    let rank = SESSIONS.fetch_add(1, Ordering::Relaxed);
    session_nonce_from(SystemTime::now(), std::process::id(), rank)
}

/// Combine les millisecondes depuis 1970 (0 avant), le processus et le rang ; injectif tant que le rang reste sous 2^22.
pub fn session_nonce_from(now: SystemTime, process: u32, rank: u64) -> u64 {
    let millis = now
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_millis())
        .unwrap_or(0);
    let millis = u64::try_from(millis).unwrap_or(u64::MAX);
    millis.wrapping_shl(22) ^ u64::from(process).wrapping_shl(6) ^ rank
}
