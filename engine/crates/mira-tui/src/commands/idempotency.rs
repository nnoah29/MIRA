//! Clés d'idempotence déterministes : un compteur de l'état, jamais d'aléa.

use mira_assembly::mira_vocabulary::IdempotencyKey;

/// Avance le compteur et rend la clé `<préfixe>-<n>` ; deux appels ne rendent jamais la même clé.
pub fn next_key(counter: &mut u64, prefix: &str) -> IdempotencyKey {
    *counter += 1;
    IdempotencyKey(format!("{prefix}-{counter}"))
}
