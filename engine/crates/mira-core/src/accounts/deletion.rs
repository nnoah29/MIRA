//! Suppression d'un compte (S14, 7.9) : secrets, caches de pièces jointes, données locales, événement ; reprise après interruption.

use std::path::Path;

use mira_vocabulary::{Account, EventKind, MiraError};

use crate::auth::CredentialVault;
use crate::events::EventSink;
use crate::maintenance::remove_account_cache_files;
use crate::storage::Database;

/// Efface les caches, les secrets puis les données du compte, et émet `AccountDeleted` ; le serveur n'est jamais contacté
pub fn erase_account(
    database: &Database,
    vault: &CredentialVault,
    events: &dyn EventSink,
    account: &Account,
    cache_directory: &Path,
) -> Result<(), MiraError> {
    remove_account_cache_files(database, &account.id, cache_directory)?;
    vault.delete_all(&account.profile, &account.id)?;
    database.write(|tx| tx.delete_account(&account.id))?;
    events.emit(EventKind::AccountDeleted {
        account: account.id.clone(),
    })?;
    Ok(())
}

/// Comptes marqués « en suppression » par une session interrompue avant la fin
fn interrupted_deletions(database: &Database) -> Result<Vec<Account>, MiraError> {
    database.read(|tx| {
        let mut found = Vec::new();
        for account in tx.list_accounts()? {
            if tx.account_is_deleting(&account.id)? {
                found.push(account);
            }
        }
        Ok(found)
    })
}

/// Achève les suppressions interrompues (au démarrage) ; rend le nombre de comptes effacés
pub fn finish_interrupted_deletions(
    database: &Database,
    vault: &CredentialVault,
    events: &dyn EventSink,
    cache_directory: &Path,
) -> Result<usize, MiraError> {
    let interrupted = interrupted_deletions(database)?;
    for account in &interrupted {
        erase_account(database, vault, events, account, cache_directory)?;
    }
    Ok(interrupted.len())
}
