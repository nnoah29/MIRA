//! Attente de la fin des passes qui servent une demande, pour clore l'opération longue.

use std::time::Duration;

use mira_vocabulary::error::codes::cancelled;
use mira_vocabulary::{ErrorCategory, MiraError};

use crate::operations::OperationHandle;
use crate::sync_service::Completion;

const CANCEL_CHECK: Duration = Duration::from_millis(100);

/// Erreur d'une demande dont la tâche a été arrêtée (pause, suppression, fermeture)
fn interrupted() -> MiraError {
    MiraError::new(
        ErrorCategory::Cancelled,
        cancelled::BY_USER,
        "la synchronisation a été interrompue",
    )
}

/// Rend la main dès que l'annulation de l'opération est demandée
async fn cancel_requested(handle: &OperationHandle) {
    while !handle.is_cancelled() {
        tokio::time::sleep(CANCEL_CHECK).await;
    }
}

/// Attend l'issue d'une passe ; une annulation met fin à l'attente sans erreur
async fn wait_one(handle: &OperationHandle, completion: Completion) -> Result<(), MiraError> {
    tokio::select! {
        outcome = completion => outcome.unwrap_or_else(|_| Err(interrupted())),
        () = cancel_requested(handle) => Ok(()),
    }
}

/// Attend toutes les passes demandées en publiant l'avancement de l'opération
pub(super) async fn await_completions(
    handle: OperationHandle,
    completions: Vec<Completion>,
) -> Result<(), MiraError> {
    let total = u64::try_from(completions.len()).unwrap_or(u64::MAX);
    handle.progress(0, total);
    let mut done = 0;
    for completion in completions {
        wait_one(&handle, completion).await?;
        done += 1;
        handle.progress(done, total);
    }
    Ok(())
}
