//! Lancement d'une tâche longue en arrière-plan, close selon son résultat.

use std::future::Future;

use mira_vocabulary::MiraError;

use super::handle::OperationHandle;

/// Exécute le travail en arrière-plan puis termine l'opération : réussite, échec ou annulation
pub fn run_in_background<F>(handle: OperationHandle, work: F)
where
    F: Future<Output = Result<(), MiraError>> + Send + 'static,
{
    tokio::spawn(async move {
        let outcome = work.await;
        handle.complete(outcome.as_ref().map(|_| ()));
    });
}
