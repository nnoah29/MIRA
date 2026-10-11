//! Changement de l'état de connexion d'un compte : écriture puis événement.

use mira_vocabulary::{Account, AccountState, EventKind, MiraError};

use crate::events::EventSink;
use crate::storage::Database;

/// Enregistre le nouvel état, émet `AccountStateChanged` et rend le compte à jour
pub fn change_account_state(
    database: &Database,
    events: &dyn EventSink,
    mut account: Account,
    target: AccountState,
) -> Result<Account, MiraError> {
    database.write(|tx| tx.set_account_state(&account.id, &target))?;
    events.emit(EventKind::AccountStateChanged {
        account: account.id.clone(),
        state: target.clone(),
    })?;
    account.state = target;
    Ok(account)
}
