//! Choix des boîtes synchronisées d'un compte (12.4) : réglage local, sans réseau.

use mira_vocabulary::error::codes::invalid_input;
use mira_vocabulary::{AccountId, ErrorCategory, EventKind, MailboxId, MiraError};

use crate::accounts::require_active_account;
use crate::api::contract::Response;
use crate::api::engine::Engine;
use crate::events::EventSink;
use crate::storage::Tx;

/// Refuse une boîte inconnue ou qui appartient à un autre compte
fn require_own_mailbox(tx: &Tx<'_>, account: &AccountId, id: &MailboxId) -> Result<(), MiraError> {
    let owned = tx
        .get_mailbox(id)?
        .is_some_and(|mailbox| &mailbox.account == account);
    if owned {
        return Ok(());
    }
    Err(MiraError::new(
        ErrorCategory::InvalidInput,
        invalid_input::PARAMETER,
        "boîte inconnue pour ce compte",
    ))
}

/// Mémorise la liste dans le compte et aligne l'indicateur de chaque boîte
fn apply_selection(
    tx: &Tx<'_>,
    account: &AccountId,
    selected: &[MailboxId],
) -> Result<(), MiraError> {
    for id in selected {
        require_own_mailbox(tx, account, id)?;
    }
    let mut record = require_active_account(tx, account)?;
    record.sync.synced_mailboxes = Some(selected.to_vec());
    tx.update_account(&record)?;
    for mut mailbox in tx.list_mailboxes(account)? {
        mailbox.sync_enabled = selected.contains(&mailbox.id);
        tx.upsert_mailbox(&mailbox)?;
    }
    Ok(())
}

/// Remplace la sélection des boîtes synchronisées, de façon atomique, puis émet `AccountUpdated`
pub(crate) fn handle_set_synced_mailboxes(
    engine: &Engine,
    account: &AccountId,
    mailboxes: Vec<MailboxId>,
) -> Result<Response, MiraError> {
    engine
        .database
        .write(|tx| apply_selection(tx, account, &mailboxes))?;
    engine.events.emit(EventKind::AccountUpdated {
        account: account.clone(),
    })?;
    Ok(Response::Done)
}
