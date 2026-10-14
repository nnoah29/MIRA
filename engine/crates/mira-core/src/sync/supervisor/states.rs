//! Transitions de l'état d'un compte (Offline, Connected, AuthRequired) et événements associés.

use mira_vocabulary::{AccountState, EventKind};

use super::runtime::AccountRuntime;

/// Passe le compte dans un nouvel état : base, `AccountStateChanged`, puis `WentOffline` ou `BackOnline`.
pub fn transition(runtime: &mut AccountRuntime, next: AccountState) {
    if runtime.state == next {
        return;
    }
    let previous = std::mem::replace(&mut runtime.state, next.clone());
    let account = runtime.ctx.account.id.clone();
    if let Err(error) = runtime
        .ctx
        .db
        .write(|tx| tx.set_account_state(&account, &next))
    {
        tracing::warn!(code = %error.code, "état du compte non enregistré");
    }
    runtime.ctx.emit(EventKind::AccountStateChanged {
        account: account.clone(),
        state: next.clone(),
    });
    match (previous, next) {
        (_, AccountState::Offline) => runtime.ctx.emit(EventKind::WentOffline { account }),
        (AccountState::Offline, AccountState::Connected) => {
            runtime.ctx.emit(EventKind::BackOnline { account });
        }
        _ => {}
    }
}
