//! Conversion entre une ligne de la table `actions` et une `PendingAction`.

use mira_vocabulary::error::codes::{local_storage, not_found};
use mira_vocabulary::{
    AccountId, ActionId, ActionState, ErrorCategory, ErrorContext, IdempotencyKey, MiraError,
    PendingAction, Timestamp,
};

use crate::storage::json_columns::from_json;

pub(super) const COLUMNS: &str = "id, account_id, kind_json, targets_json, idempotency_key, state, attempts, last_error, created_at, not_before";

/// Ligne brute de la table `actions`, dans l'ordre de `COLUMNS`.
pub(super) struct RawAction {
    id: String,
    account: String,
    kind_json: String,
    targets_json: String,
    idempotency_key: String,
    state: String,
    attempts: u32,
    last_error: Option<String>,
    created_at: i64,
    not_before: i64,
}

impl RawAction {
    /// Lit une ligne sélectionnée avec `COLUMNS`
    pub(super) fn from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get(0)?,
            account: row.get(1)?,
            kind_json: row.get(2)?,
            targets_json: row.get(3)?,
            idempotency_key: row.get(4)?,
            state: row.get(5)?,
            attempts: row.get(6)?,
            last_error: row.get(7)?,
            created_at: row.get(8)?,
            not_before: row.get(9)?,
        })
    }

    /// Reconstruit l'action du domaine depuis la ligne
    pub(super) fn into_action(self) -> Result<PendingAction, MiraError> {
        Ok(PendingAction {
            id: ActionId::from_raw(self.id),
            account: AccountId::from_raw(self.account),
            kind: from_json(&self.kind_json)?,
            targets: from_json(&self.targets_json)?,
            idempotency_key: IdempotencyKey(self.idempotency_key),
            state: state_from_name(&self.state)?,
            attempts: self.attempts,
            last_error: self.last_error,
            created_at: Timestamp::from_millis(self.created_at),
            not_before: Timestamp::from_millis(self.not_before),
        })
    }
}

/// Donne le nom stocké d'un état d'action
pub(super) fn state_name(state: ActionState) -> &'static str {
    match state {
        ActionState::Pending => "pending",
        ActionState::InProgress => "in_progress",
        ActionState::Succeeded => "succeeded",
        ActionState::Failed => "failed",
        ActionState::Cancelled => "cancelled",
        ActionState::Uncertain => "uncertain",
    }
}

/// Relit un état d'action depuis son nom stocké
fn state_from_name(name: &str) -> Result<ActionState, MiraError> {
    match name {
        "pending" => Ok(ActionState::Pending),
        "in_progress" => Ok(ActionState::InProgress),
        "succeeded" => Ok(ActionState::Succeeded),
        "failed" => Ok(ActionState::Failed),
        "cancelled" => Ok(ActionState::Cancelled),
        "uncertain" => Ok(ActionState::Uncertain),
        _ => Err(MiraError::new(
            ErrorCategory::LocalStorage,
            local_storage::CORRUPTED,
            "état d'action inconnu en base",
        )),
    }
}

/// Erreur « action inconnue » portant l'identifiant visé
pub(super) fn action_not_found(id: &ActionId) -> MiraError {
    MiraError::new(
        ErrorCategory::NotFound,
        not_found::ACTION,
        "action inconnue",
    )
    .with_context(ErrorContext {
        action: Some(id.clone()),
        ..ErrorContext::default()
    })
}
