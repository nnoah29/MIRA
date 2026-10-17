//! Tâche de fond d'un compte : attend du travail prêt, se connecte, rejoue la file (10.7).

use std::sync::Arc;
use std::time::Duration;

use mira_vocabulary::{Account, AccountId, AccountState, ActionState, MiraError};

use super::shared::ReplayShared;
use super::work::{settle_failure, work_online};

const IDLE_PATIENCE: Duration = Duration::from_millis(250);
const OFFLINE_PATIENCE: Duration = Duration::from_secs(1);

/// Ce que la tâche fait après un tour.
enum Tick {
    Gone,
    Wait(Duration),
}

/// Boucle d'un compte : rend son identifiant quand le compte disparaît ou que le moteur s'arrête
pub(crate) async fn run_account_replayer(shared: Arc<ReplayShared>, id: AccountId) -> AccountId {
    let mut recovered = false;
    while !shared.is_stopping() {
        match tick(&shared, &id, &mut recovered).await {
            Tick::Gone => break,
            Tick::Wait(patience) => shared.wait(patience).await,
        }
    }
    id
}

/// Un tour : rien si le compte est en pause ou sans travail prêt, sinon une passe en ligne
async fn tick(shared: &Arc<ReplayShared>, id: &AccountId, recovered: &mut bool) -> Tick {
    let gate = shared.gates.gate(id);
    let _busy = gate.lock().await;
    let account = match load_active(shared, id) {
        Ok(Some(account)) => account,
        Ok(None) => return Tick::Gone,
        Err(_) => return Tick::Wait(IDLE_PATIENCE),
    };
    let replayable = !matches!(
        account.state,
        AccountState::Paused | AccountState::AuthRequired
    );
    if !replayable || !matches!(has_work(shared, &account, recovered), Ok(true)) {
        return Tick::Wait(IDLE_PATIENCE);
    }
    match work_online(shared, &account, recovered).await {
        Ok(()) => Tick::Wait(IDLE_PATIENCE),
        Err(error) => {
            let _ = settle_failure(shared, &account, &error);
            Tick::Wait(OFFLINE_PATIENCE)
        }
    }
}

/// Relit le compte, sauf s'il n'existe plus ou est en cours de suppression
fn load_active(shared: &ReplayShared, id: &AccountId) -> Result<Option<Account>, MiraError> {
    shared.database.read(|tx| match tx.get_account(id)? {
        Some(account) if !tx.account_is_deleting(id)? => Ok(Some(account)),
        Some(_) | None => Ok(None),
    })
}

/// Dit s'il y a une action interrompue à régler ou une action prête à partir
fn has_work(
    shared: &ReplayShared,
    account: &Account,
    recovered: &mut bool,
) -> Result<bool, MiraError> {
    let now = shared.config.clock.now();
    let (interrupted, ready) = shared.database.read(|tx| {
        let interrupted = tx.list_actions(&account.id, &[ActionState::InProgress])?;
        let ready = tx.next_ready_action(&account.id, now)?;
        Ok((interrupted.len(), ready.is_some()))
    })?;
    *recovered = interrupted == 0;
    Ok(interrupted > 0 || ready)
}
