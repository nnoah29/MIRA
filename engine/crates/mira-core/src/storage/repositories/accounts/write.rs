//! Écritures sur les comptes : insertion, mise à jour, état, suppression.

use mira_vocabulary::{Account, AccountId, AccountState, MiraError};
use rusqlite::params;

use super::rows::{AccountJson, account_not_found, auth_kind_name};
use crate::storage::Tx;
use crate::storage::errors::storage_error;
use crate::storage::json_columns::to_json;

impl Tx<'_> {
    /// Insère un nouveau compte
    pub fn insert_account(&self, account: &Account) -> Result<(), MiraError> {
        let json = AccountJson::of(account)?;
        self.sql()
            .execute(
                "INSERT INTO accounts (id, profile_id, address, display_name, provider_id, auth_kind, servers_json, sync_json, state_json, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                params![
                    account.id.as_str(),
                    account.profile.as_str(),
                    account.address.as_str(),
                    account.display_name,
                    account.provider.as_str(),
                    auth_kind_name(account.auth),
                    json.servers,
                    json.sync,
                    json.state,
                    account.created_at.as_millis(),
                ],
            )
            .map_err(storage_error)?;
        Ok(())
    }

    /// Met à jour tous les champs modifiables d'un compte (pas sa date de création)
    pub fn update_account(&self, account: &Account) -> Result<(), MiraError> {
        let json = AccountJson::of(account)?;
        let changed = self
            .sql()
            .execute(
                "UPDATE accounts SET profile_id = ?2, address = ?3, display_name = ?4, provider_id = ?5, auth_kind = ?6, servers_json = ?7, sync_json = ?8, state_json = ?9 WHERE id = ?1",
                params![
                    account.id.as_str(),
                    account.profile.as_str(),
                    account.address.as_str(),
                    account.display_name,
                    account.provider.as_str(),
                    auth_kind_name(account.auth),
                    json.servers,
                    json.sync,
                    json.state,
                ],
            )
            .map_err(storage_error)?;
        require_change(changed, &account.id)
    }

    /// Change l'état de connexion d'un compte
    pub fn set_account_state(&self, id: &AccountId, state: &AccountState) -> Result<(), MiraError> {
        let changed = self
            .sql()
            .execute(
                "UPDATE accounts SET state_json = ?2 WHERE id = ?1",
                params![id.as_str(), to_json(state)?],
            )
            .map_err(storage_error)?;
        require_change(changed, id)
    }

    /// Marque un compte « en suppression » pour refuser toute nouvelle opération
    pub fn mark_account_deleting(&self, id: &AccountId) -> Result<(), MiraError> {
        let changed = self
            .sql()
            .execute(
                "UPDATE accounts SET deleting = 1 WHERE id = ?1",
                [id.as_str()],
            )
            .map_err(storage_error)?;
        require_change(changed, id)
    }

    /// Supprime un compte et, en cascade, tout ce qui en dépend (y compris l'index de recherche)
    pub fn delete_account(&self, id: &AccountId) -> Result<(), MiraError> {
        self.sql()
            .execute(
                "DELETE FROM messages_fts WHERE message_id IN (SELECT id FROM messages WHERE account_id = ?1)",
                [id.as_str()],
            )
            .map_err(storage_error)?;
        let changed = self
            .sql()
            .execute("DELETE FROM accounts WHERE id = ?1", [id.as_str()])
            .map_err(storage_error)?;
        require_change(changed, id)
    }
}

/// Échoue avec « compte inconnu » si l'écriture n'a touché aucune ligne
fn require_change(changed: usize, id: &AccountId) -> Result<(), MiraError> {
    if changed == 0 {
        return Err(account_not_found(id));
    }
    Ok(())
}
