//! État de synchronisation d'un compte : base locale, rapport de la tâche et journal des événements.

use mira_vocabulary::{Account, MiraError};

use crate::api::contract::AccountSyncStatus;
use crate::api::engine::Engine;

/// Construit l'état du compte ; sans tâche (pause, désactivé), le dernier succès vient du journal des événements
pub(super) fn status_of(
    engine: &Engine,
    account: &Account,
) -> Result<AccountSyncStatus, MiraError> {
    let (state, pending_actions, journal) = engine.database.read(|tx| {
        let fresh = tx.get_account(&account.id)?;
        Ok((
            fresh.map_or_else(|| account.state.clone(), |known| known.state),
            tx.pending_action_count(&account.id)?,
            tx.last_sync_finished_at(&account.id)?,
        ))
    })?;
    let report = engine.sync.report_of(&account.id).unwrap_or_default();
    Ok(AccountSyncStatus {
        account: account.id.clone(),
        state,
        syncing: report.running,
        last_synced_at: report.last_success.or(journal),
        last_error: report.last_error,
        mailboxes_in_progress: report.mailboxes,
        pending_actions,
    })
}
