//! Lectures sur la file d'actions.

use mira_vocabulary::{
    AccountId, ActionId, ActionState, IdempotencyKey, MiraError, PendingAction, Timestamp,
};
use rusqlite::{OptionalExtension, Params};

use super::rows::{COLUMNS, RawAction, state_name};
use crate::storage::Tx;
use crate::storage::errors::storage_error;

impl Tx<'_> {
    /// Relit une action par son identifiant
    pub fn get_action(&self, id: &ActionId) -> Result<Option<PendingAction>, MiraError> {
        self.query_one_action("WHERE id = ?1", [id.as_str()])
    }

    /// Retrouve l'action d'un compte déjà enregistrée sous une clé d'idempotence
    pub fn find_action_by_idempotency_key(
        &self,
        account: &AccountId,
        key: &IdempotencyKey,
    ) -> Result<Option<PendingAction>, MiraError> {
        self.query_one_action(
            "WHERE account_id = ?1 AND idempotency_key = ?2",
            [account.as_str(), key.0.as_str()],
        )
    }

    /// Donne la plus ancienne action en attente prête à partir de `now`, selon l'ordre de la file
    pub fn next_ready_action(
        &self,
        account: &AccountId,
        now: Timestamp,
    ) -> Result<Option<PendingAction>, MiraError> {
        self.query_one_action(
            "WHERE account_id = ?1 AND state = 'pending' AND not_before <= ?2 ORDER BY seq LIMIT 1",
            rusqlite::params![account.as_str(), now.as_millis()],
        )
    }

    /// Liste les actions d'un compte dans l'ordre de la file, filtrées par états (vide : tous)
    pub fn list_actions(
        &self,
        account: &AccountId,
        states: &[ActionState],
    ) -> Result<Vec<PendingAction>, MiraError> {
        let mut statement = self
            .sql()
            .prepare(&format!(
                "SELECT {COLUMNS} FROM actions WHERE account_id = ?1 ORDER BY seq"
            ))
            .map_err(storage_error)?;
        let raws = statement
            .query_map([account.as_str()], RawAction::from_row)
            .map_err(storage_error)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(storage_error)?;
        let actions = raws
            .into_iter()
            .map(RawAction::into_action)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(actions
            .into_iter()
            .filter(|action| states.is_empty() || states.contains(&action.state))
            .collect())
    }

    /// Compte les actions d'un compte encore en attente
    pub fn pending_action_count(&self, account: &AccountId) -> Result<u64, MiraError> {
        let count: i64 = self
            .sql()
            .query_row(
                "SELECT COUNT(*) FROM actions WHERE account_id = ?1 AND state = ?2",
                [account.as_str(), state_name(ActionState::Pending)],
                |row| row.get(0),
            )
            .map_err(storage_error)?;
        Ok(u64::try_from(count).unwrap_or(0))
    }

    /// Lit la première action qui satisfait la clause donnée
    fn query_one_action(
        &self,
        clause: &str,
        params: impl Params,
    ) -> Result<Option<PendingAction>, MiraError> {
        let raw = self
            .sql()
            .query_row(
                &format!("SELECT {COLUMNS} FROM actions {clause}"),
                params,
                RawAction::from_row,
            )
            .optional()
            .map_err(storage_error)?;
        raw.map(RawAction::into_action).transpose()
    }
}
