//! Suppression définitive : `\Deleted` puis purge ciblée sur chaque apparition retrouvée.

use mira_vocabulary::MiraError;

use super::locate::{Scope, locate, wanted_by_mailbox};
use super::purge::{effective, purge_selected};
use crate::actions::replay::job::ReplayJob;
use crate::imap::ImapClient;
use mira_vocabulary::ProviderQuirks;

/// Supprime les messages encore présents ; un message déjà disparu est un objectif atteint
pub(super) async fn delete_permanently(
    client: &mut ImapClient,
    quirks: &ProviderQuirks,
    job: &ReplayJob,
) -> Result<(), MiraError> {
    let capabilities = effective(client, quirks);
    for (mailbox, wanted) in wanted_by_mailbox(&job.messages, &Scope::All) {
        let located = locate(client, &mailbox, &wanted).await?;
        let uids: Vec<u32> = located.found.iter().map(|(_, uid)| *uid).collect();
        purge_selected(client, &capabilities, &uids).await?;
    }
    Ok(())
}
