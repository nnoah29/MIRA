//! Superviseur : une tâche de rejeu par compte, démarrée quand le compte apparaît.

use std::collections::BTreeSet;
use std::sync::Arc;
use std::time::Duration;

use mira_vocabulary::{AccountId, MiraError};
use tokio::task::JoinSet;

use super::account_task::run_account_replayer;
use super::shared::ReplayShared;

const SCAN_PATIENCE: Duration = Duration::from_millis(250);

/// Boucle du superviseur ; en la détruisant, toutes les tâches de compte sont arrêtées avec elle
pub(crate) async fn supervise(shared: Arc<ReplayShared>) {
    let mut tasks: JoinSet<AccountId> = JoinSet::new();
    let mut running: BTreeSet<AccountId> = BTreeSet::new();
    while !shared.is_stopping() {
        while let Some(finished) = tasks.try_join_next() {
            if let Ok(id) = finished {
                running.remove(&id);
            }
        }
        if let Ok(known) = active_accounts(&shared) {
            for id in known {
                if running.insert(id.clone()) {
                    tasks.spawn(run_account_replayer(Arc::clone(&shared), id));
                }
            }
        }
        shared.wait(SCAN_PATIENCE).await;
    }
}

/// Identifiants des comptes qui ne sont pas en cours de suppression
fn active_accounts(shared: &ReplayShared) -> Result<Vec<AccountId>, MiraError> {
    shared.database.read(|tx| {
        let mut active = Vec::new();
        for account in tx.list_accounts()? {
            if !tx.account_is_deleting(&account.id)? {
                active.push(account.id);
            }
        }
        Ok(active)
    })
}
