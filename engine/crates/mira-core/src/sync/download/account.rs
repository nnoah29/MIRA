//! Descente d'un compte (D-20) : boîtes, en-têtes de chaque boîte (réception d'abord), corps préchargés.

use mira_vocabulary::{ErrorCategory, EventKind, LogLevel, Mailbox, MailboxRole, MiraError};

use super::mailbox_list::sync_mailbox_list;
use super::order::sync_order;
use super::orphans::delete_orphans;
use super::prefetch::prefetch_bodies;
use crate::sync::session::SyncSession;
use crate::sync::strategy::sync_mailbox;

/// Issue de la synchronisation d'une boîte : faite, ou ignorée sur refus du serveur.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum MailboxOutcome {
    Synced,
    Skipped,
}

/// Synchronise une boîte en la signalant au rapport du compte pendant le travail.
pub(super) async fn sync_one(
    session: &mut SyncSession,
    mailbox: &Mailbox,
) -> Result<MailboxOutcome, MiraError> {
    session.report.mailbox_started(&mailbox.id);
    let outcome = sync_and_prefetch(session, mailbox).await;
    session.report.mailbox_ended(&mailbox.id);
    outcome
}

/// Synchronise une boîte puis, pour la réception, précharge les corps ; un refus du serveur n'arrête pas les autres boîtes.
async fn sync_and_prefetch(
    session: &mut SyncSession,
    mailbox: &Mailbox,
) -> Result<MailboxOutcome, MiraError> {
    match sync_mailbox(session, mailbox).await {
        Err(error) if error.category == ErrorCategory::Server => {
            let text = format!("boîte {} ignorée : {}", mailbox.name, error.code);
            session.ctx.diagnose(LogLevel::Warn, "sync.mailbox", &text);
            return Ok(MailboxOutcome::Skipped);
        }
        other => other?,
    }
    if mailbox.role == MailboxRole::Inbox {
        prefetch_bodies(session, mailbox).await?;
    }
    Ok(MailboxOutcome::Synced)
}

/// Les étapes de la descente, sans les événements de début et de fin.
pub(super) async fn run_steps(session: &mut SyncSession) -> Result<(), MiraError> {
    let mailboxes = sync_order(sync_mailbox_list(session).await?);
    let mut complete = true;
    for mailbox in &mailboxes {
        complete &= sync_one(session, mailbox).await? == MailboxOutcome::Synced;
    }
    if !complete {
        let text = "boîte ignorée : les orphelins sont gardés jusqu'à une passe complète";
        session.ctx.diagnose(LogLevel::Info, "sync.orphans", text);
        return Ok(());
    }
    delete_orphans(&session.ctx)
}

/// Encadre une passe par `SyncStarted` puis `SyncFinished` ou `SyncFailed`.
pub async fn run_pass(
    session: &mut SyncSession,
    work: impl AsyncFnOnce(&mut SyncSession) -> Result<(), MiraError>,
) -> Result<(), MiraError> {
    session.begin_pass();
    let (account, operation) = (session.ctx.account.id.clone(), session.operation.clone());
    session.report.pass_started();
    session.ctx.emit(EventKind::SyncStarted {
        account: account.clone(),
        operation: operation.clone(),
    });
    let outcome = work(session).await;
    session.report.pass_ended(session.ctx.clock.now(), &outcome);
    session.ctx.emit(match &outcome {
        Ok(()) => EventKind::SyncFinished { account, operation },
        Err(error) => EventKind::SyncFailed {
            account,
            operation,
            code: error.code.clone(),
        },
    });
    session.ctx.apply_event_retention();
    outcome
}

/// Synchronise tout le compte : liste des boîtes, chaque boîte, ménage des orphelins.
pub async fn sync_account(session: &mut SyncSession) -> Result<(), MiraError> {
    run_pass(session, async |pass| run_steps(pass).await).await
}
