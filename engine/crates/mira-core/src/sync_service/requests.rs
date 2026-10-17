//! Demandes de la façade au service : synchroniser, resynchroniser, remonter la plage, lire le rapport.

use mira_vocabulary::error::codes::{incompatible_state, internal};
use mira_vocabulary::{
    Account, AccountId, AccountState, ErrorCategory, ErrorContext, MailboxId, MiraError,
    OperationId,
};
use tokio::sync::oneshot;

use super::accounts::context_for;
use super::service::{ActiveSync, SyncService};
use crate::operations::OperationRegistry;
use crate::sync::{RequestScope, SyncReport, start_extend_window};

/// Attente de la fin d'une demande de synchronisation
pub(crate) type Completion = oneshot::Receiver<Result<(), MiraError>>;

/// Erreur d'un compte en pause : il faut le reprendre d'abord
fn account_paused(account: &AccountId) -> MiraError {
    let context = ErrorContext {
        account: Some(account.clone()),
        ..ErrorContext::default()
    };
    MiraError::new(
        ErrorCategory::IncompatibleState,
        incompatible_state::ACCOUNT_PAUSED,
        "le compte est en pause",
    )
    .with_context(context)
}

/// Dit que la tâche du compte devrait exister et ne répond pas
fn task_missing() -> MiraError {
    MiraError::new(
        ErrorCategory::Internal,
        internal::BUG,
        "la tâche de synchronisation du compte est introuvable",
    )
}

impl SyncService {
    /// Service actif et tâche du compte lancée, ou erreur (désactivé, compte en pause)
    fn runnable(&self, account: &Account) -> Result<&ActiveSync, MiraError> {
        let active = self.require_active()?;
        if account.state == AccountState::Paused {
            return Err(account_paused(&account.id));
        }
        self.start_account(account)?;
        Ok(active)
    }

    /// Met une demande dans la file du compte ; l'attente rend l'issue de la passe qui la sert
    pub(crate) fn submit(
        &self,
        account: &Account,
        scope: RequestScope,
        operation: Option<OperationId>,
    ) -> Result<Completion, MiraError> {
        let active = self.runnable(account)?;
        let (ticket, completion) = oneshot::channel();
        let accepted = active
            .supervisor
            .submit(&account.id, scope, Some(ticket), operation);
        accepted.then_some(completion).ok_or_else(task_missing)
    }

    /// Rapport d'activité du compte, s'il a une tâche
    pub(crate) fn report_of(&self, account: &AccountId) -> Option<SyncReport> {
        let active = self.active.as_ref()?;
        active.supervisor.report_of(account)
    }

    /// Lance « remonter plus loin » pour une boîte du compte
    pub(crate) fn start_extension(
        &self,
        account: &Account,
        operations: &OperationRegistry,
        mailbox: MailboxId,
        days: u32,
    ) -> Result<OperationId, MiraError> {
        let active = self.runnable(account)?;
        let budget = active.supervisor.budget_of(&account.id);
        let budget = budget.ok_or_else(task_missing)?;
        let ctx = context_for(active, account)?;
        let credentials = active.credentials.clone();
        Ok(start_extend_window(
            ctx,
            budget,
            credentials,
            operations,
            mailbox,
            days,
        ))
    }
}
