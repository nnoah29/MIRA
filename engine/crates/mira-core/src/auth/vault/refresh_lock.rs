//! Verrou de rafraîchissement par compte : deux renouvellements d'un même jeton ne se doublent pas.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

use mira_vocabulary::{AccountId, ProfileId};

type LockTable = Mutex<HashMap<String, Arc<tokio::sync::Mutex<()>>>>;

static LOCKS: OnceLock<LockTable> = OnceLock::new();

/// Rend le verrou asynchrone propre à ce compte, créé à la première demande
pub(super) fn refresh_lock(
    profile: &ProfileId,
    account: &AccountId,
) -> Arc<tokio::sync::Mutex<()>> {
    let key = format!("{}/{}", profile.as_str(), account.as_str());
    let mut table = LOCKS
        .get_or_init(LockTable::default)
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    Arc::clone(table.entry(key).or_default())
}
