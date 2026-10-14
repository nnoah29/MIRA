//! Comptes et portées visés par une demande de synchronisation.

use mira_vocabulary::{Account, AccountState, MailboxId, MiraError};

use crate::accounts::require_active_account;
use crate::api::contract::SyncScope;
use crate::api::engine::Engine;
use crate::storage::repositories::mailboxes::mailbox_not_found;
use crate::sync::RequestScope;

/// Un compte et ce qu'on lui demande de synchroniser.
#[derive(Debug)]
pub(super) struct Target {
    pub(super) account: Account,
    pub(super) scope: RequestScope,
}

/// Compte actif qui possède la boîte, ou `not_found.mailbox`
pub(super) fn mailbox_owner(engine: &Engine, mailbox: &MailboxId) -> Result<Account, MiraError> {
    engine.database.read(|tx| {
        let found = tx.get_mailbox(mailbox)?;
        let known = found.ok_or_else(|| mailbox_not_found(mailbox))?;
        require_active_account(tx, &known.account)
    })
}

/// Comptes actifs et non en pause, pour « tous les comptes »
fn syncable_accounts(engine: &Engine) -> Result<Vec<Account>, MiraError> {
    engine.database.read(|tx| {
        let mut kept = Vec::new();
        for account in tx.list_accounts()? {
            let paused = account.state == AccountState::Paused;
            if !paused && !tx.account_is_deleting(&account.id)? {
                kept.push(account);
            }
        }
        Ok(kept)
    })
}

/// Traduit la portée de la requête en comptes et portées de demande
pub(super) fn resolve_targets(
    engine: &Engine,
    scope: &SyncScope,
) -> Result<Vec<Target>, MiraError> {
    match scope {
        SyncScope::All => Ok(syncable_accounts(engine)?
            .into_iter()
            .map(|account| Target {
                account,
                scope: RequestScope::Account,
            })
            .collect()),
        SyncScope::Account { account } => {
            let found = engine
                .database
                .read(|tx| require_active_account(tx, account))?;
            let scope = RequestScope::Account;
            Ok(vec![Target {
                account: found,
                scope,
            }])
        }
        SyncScope::Mailbox { mailbox } => {
            let account = mailbox_owner(engine, mailbox)?;
            let scope = RequestScope::Mailbox(mailbox.clone());
            Ok(vec![Target { account, scope }])
        }
    }
}
