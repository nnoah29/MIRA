//! Horloge du faux serveur.

use std::time::{SystemTime, UNIX_EPOCH};

/// Secondes écoulées depuis l'époque Unix à l'instant présent
pub(crate) fn now_epoch() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs() as i64)
        .unwrap_or(0)
}
