//! Passe déclenchée par des demandes : resynchronisations forcées, puis compte entier ou boîtes demandées.

use mira_vocabulary::{MailboxId, MiraError};

use super::account::{MailboxOutcome, run_pass, run_steps, sync_one};
use crate::sync::requests::TakenRequests;
use crate::sync::resync::force_resync;
use crate::sync::session::SyncSession;

/// Synchronise une boîte connue du miroir ; une boîte disparue entre-temps est ignorée.
async fn sync_listed(session: &mut SyncSession, mailbox: &MailboxId) -> Result<(), MiraError> {
    let found = session.ctx.db.read(|tx| tx.get_mailbox(mailbox))?;
    match found {
        Some(known) => sync_one(session, &known).await.map(|_: MailboxOutcome| ()),
        None => Ok(()),
    }
}

/// Étapes d'une passe demandée : resynchronisations, puis le compte entier ou les seules boîtes demandées.
async fn run_requested(session: &mut SyncSession, taken: &TakenRequests) -> Result<(), MiraError> {
    for mailbox in taken.resyncs() {
        force_resync(session, &mailbox).await?;
    }
    if taken.wants_account() {
        return run_steps(session).await;
    }
    for mailbox in taken.mailboxes() {
        sync_listed(session, &mailbox).await?;
    }
    Ok(())
}

/// Sert les demandes en une seule passe, qui porte l'identifiant d'opération du premier demandeur.
pub async fn sync_requested(
    session: &mut SyncSession,
    taken: &TakenRequests,
) -> Result<(), MiraError> {
    if let Some(operation) = &taken.operation {
        session.force_operation(operation.clone());
    }
    run_pass(session, async |pass| run_requested(pass, taken).await).await
}
