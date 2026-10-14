//! Délai avant la prochaine tentative d'une action qui a échoué de façon passagère.

const BASE_MILLIS: i64 = 5_000;
const MAX_MILLIS: i64 = 600_000;

/// Attente après `attempts` échecs : 5 s, 10 s, 20 s… plafonnée à 10 minutes
pub(crate) fn backoff_millis(attempts: u32) -> i64 {
    let doublings = attempts.saturating_sub(1).min(10);
    (BASE_MILLIS << doublings).min(MAX_MILLIS)
}
