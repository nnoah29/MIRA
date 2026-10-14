//! Déplacement vers une boîte : MOVE si possible, sinon COPY + `\Deleted` + UID EXPUNGE (9.9, 10.14).

use std::collections::BTreeSet;

use mira_vocabulary::{MessageId, MiraError, ProviderQuirks};

use super::locate::{Scope, locate, wanted_by_mailbox};
use super::locate_require::require_any_handled;
use super::purge::{effective, purge_selected};
use crate::actions::replay::job::ReplayJob;
use crate::capabilities::{EffectiveCapabilities, MoveMethod, choose_move};
use crate::imap::ImapClient;

/// UID d'une boîte source à déplacer, et UID dont la copie existe déjà dans la destination.
#[derive(Debug, Default)]
struct GroupPlan {
    transfer: Vec<u32>,
    purge: Vec<u32>,
}

/// Déplace chaque message vers `destination` ; vérifie d'abord ce que le serveur contient déjà (inv. 18)
pub(super) async fn move_to(
    client: &mut ImapClient,
    quirks: &ProviderQuirks,
    job: &ReplayJob,
    destination: &str,
) -> Result<(), MiraError> {
    let capabilities = effective(client, quirks);
    let landed = already_in_destination(client, job, destination).await?;
    let mut moved = BTreeSet::new();
    let scope = Scope::Except(destination.to_string());
    for (source, wanted) in wanted_by_mailbox(&job.messages, &scope) {
        let located = locate(client, &source, &wanted).await?;
        let plan = plan_group(located.found, &landed, &mut moved);
        purge_selected(client, &capabilities, &plan.purge).await?;
        transfer(client, &capabilities, &plan.transfer, destination).await?;
    }
    moved.extend(landed);
    require_any_handled(&job.messages, &moved)
}

/// Sépare les messages à déplacer de ceux déjà arrivés ; un message n'est déplacé qu'une fois
fn plan_group(
    found: Vec<(MessageId, u32)>,
    landed: &BTreeSet<MessageId>,
    moved: &mut BTreeSet<MessageId>,
) -> GroupPlan {
    let mut plan = GroupPlan::default();
    for (message, uid) in found {
        if landed.contains(&message) {
            plan.purge.push(uid);
        } else if moved.insert(message) {
            plan.transfer.push(uid);
        }
    }
    plan
}

/// Messages que la boîte de destination contient déjà, retrouvés par `Message-ID`
async fn already_in_destination(
    client: &mut ImapClient,
    job: &ReplayJob,
    destination: &str,
) -> Result<BTreeSet<MessageId>, MiraError> {
    client.select(destination, None).await?;
    let mut landed = BTreeSet::new();
    for message in &job.messages {
        let Some(header) = &message.header else {
            continue;
        };
        if !client.uids_with_message_id(header).await?.is_empty() {
            landed.insert(message.id.clone());
        }
    }
    Ok(landed)
}

/// Déplace les UID de la boîte sélectionnée : MOVE, ou copie puis purge
async fn transfer(
    client: &mut ImapClient,
    capabilities: &EffectiveCapabilities,
    uids: &[u32],
    destination: &str,
) -> Result<(), MiraError> {
    if uids.is_empty() {
        return Ok(());
    }
    match choose_move(capabilities) {
        MoveMethod::Move => client.move_messages(uids, destination).await,
        MoveMethod::CopyThenDelete => {
            client.copy(uids, destination).await?;
            purge_selected(client, capabilities, uids).await
        }
    }
}
