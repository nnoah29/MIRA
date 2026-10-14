//! Ce que le rejeu relit en base pour une action : intention, messages visés, boîte visée.

use mira_vocabulary::{MailboxId, MailboxRole, MessageId, MiraError, PendingAction};

use super::context::ReplayContext;
use crate::actions::model::LocalIntent;
use crate::actions::record::load_record;
use crate::actions::targets::{AppearanceTarget, ServerTargets, load_server_targets};
use crate::storage::Tx;

/// Un message visé : son `Message-ID` d'en-tête (pour le retrouver) et ses apparitions sur le serveur.
#[derive(Debug, Clone)]
pub(crate) struct ServerMessage {
    pub id: MessageId,
    pub header: Option<String>,
    pub appearances: Vec<AppearanceTarget>,
}

/// Boîte visée par l'action, avec le nom qu'elle porte sur le serveur.
#[derive(Debug, Clone)]
pub(crate) struct NamedMailbox {
    pub name: String,
    pub role: MailboxRole,
}

/// Tout ce qu'il faut pour exécuter une action sur le serveur sans relire le miroir.
#[derive(Debug, Clone)]
pub(crate) struct ReplayJob {
    pub intent: LocalIntent,
    pub messages: Vec<ServerMessage>,
    pub mailbox: Option<NamedMailbox>,
}

/// Relit l'intention mémorisée, les cibles serveur et la boîte visée de l'action
pub(crate) fn load_job(
    ctx: &ReplayContext<'_>,
    action: &PendingAction,
) -> Result<ReplayJob, MiraError> {
    ctx.queue.db.read(|tx| {
        let record = load_record(tx, &action.id)?;
        let intent = record.map_or(LocalIntent::NoEffect, |found| found.intent);
        let targets = load_server_targets(tx, &action.id)?;
        Ok(ReplayJob {
            messages: server_messages(tx, &targets)?,
            mailbox: named_mailbox(tx, &intent, &targets)?,
            intent,
        })
    })
}

/// Associe à chaque message visé son `Message-ID` d'en-tête, lu dans le miroir
fn server_messages(tx: &Tx<'_>, targets: &ServerTargets) -> Result<Vec<ServerMessage>, MiraError> {
    let mut messages = Vec::new();
    for target in &targets.messages {
        let summary = tx.get_message(&target.message)?;
        messages.push(ServerMessage {
            id: target.message.clone(),
            header: summary.and_then(|found| found.message_id_header),
            appearances: target.appearances.clone(),
        });
    }
    Ok(messages)
}

/// Boîte que l'intention désigne, s'il y en a une
fn intent_mailbox(intent: &LocalIntent) -> Option<&MailboxId> {
    match intent {
        LocalIntent::MoveTo { mailbox }
        | LocalIntent::DropFrom { mailbox }
        | LocalIntent::AddTo { mailbox }
        | LocalIntent::RenameMailbox { mailbox, .. }
        | LocalIntent::DeleteMailbox { mailbox }
        | LocalIntent::EmptyMailbox { mailbox } => Some(mailbox),
        LocalIntent::CreateMailbox { .. }
        | LocalIntent::SetSeen { .. }
        | LocalIntent::SetFlagged { .. }
        | LocalIntent::MarkDeleted
        | LocalIntent::NoEffect => None,
    }
}

/// Nom de la boîte visée : celui relevé à la soumission, sinon celui du miroir
fn named_mailbox(
    tx: &Tx<'_>,
    intent: &LocalIntent,
    targets: &ServerTargets,
) -> Result<Option<NamedMailbox>, MiraError> {
    let Some(id) = intent_mailbox(intent) else {
        return Ok(None);
    };
    let mirrored = tx.get_mailbox(id)?;
    let role = mirrored
        .as_ref()
        .map_or(MailboxRole::Other, |found| found.role);
    let captured = targets
        .mailboxes
        .iter()
        .find(|target| &target.mailbox == id);
    let name = captured
        .map(|target| target.name.clone())
        .or_else(|| mirrored.map(|found| found.name));
    Ok(name.map(|name| NamedMailbox { name, role }))
}
