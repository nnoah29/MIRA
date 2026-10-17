//! Lancement de « remonter plus loin » comme tâche longue : identifiant d'opération, progression, annulation.

use std::sync::Arc;

use mira_vocabulary::{MailboxId, MiraError, OperationId};

use super::extend::extend_window;
use crate::operations::{OperationHandle, OperationRegistry};
use crate::sync::context::SyncContext;
use crate::sync::session::{ConnectionBudget, CredentialProvider, SyncSession, open_connection};

/// Ouvre une connexion dans le budget du compte puis étend la plage de la boîte.
async fn run_extension(
    ctx: SyncContext,
    budget: ConnectionBudget,
    credentials: Arc<dyn CredentialProvider>,
    handle: &OperationHandle,
    mailbox: MailboxId,
    extra_days: u32,
) -> Result<(), MiraError> {
    let permit = budget.acquire_explicit().await?;
    let connection = open_connection(&ctx, permit, credentials.as_ref()).await?;
    let mut session = SyncSession::new(ctx, connection);
    extend_window(&mut session, handle, &mailbox, extra_days)
        .await
        .map(|_| ())
}

/// Démarre l'extension en arrière-plan et rend l'identifiant de l'opération (annulable via le registre).
pub fn start_extend_window(
    ctx: SyncContext,
    budget: ConnectionBudget,
    credentials: Arc<dyn CredentialProvider>,
    operations: &OperationRegistry,
    mailbox: MailboxId,
    extra_days: u32,
) -> OperationId {
    let handle = operations.start("extend_window");
    let id = handle.id().clone();
    tokio::spawn(async move {
        let outcome = run_extension(ctx, budget, credentials, &handle, mailbox, extra_days).await;
        handle.complete(outcome.as_ref().map(|_| ()));
    });
    id
}
