//! Drapeaux « lu » et « suivi » posés sur toutes les apparitions du message (D-14).

use std::collections::BTreeSet;

use mira_vocabulary::MiraError;

use super::locate::{Scope, locate, wanted_by_mailbox};
use super::locate_require::require_any_handled;
use crate::actions::replay::job::ReplayJob;
use crate::imap::{ImapClient, ImapFlag};

/// Pose ou retire le drapeau sur chaque apparition retrouvée ; échec définitif si le message a disparu
pub(super) async fn set_flag(
    client: &mut ImapClient,
    job: &ReplayJob,
    flag: ImapFlag,
    on: bool,
) -> Result<(), MiraError> {
    let mut handled = BTreeSet::new();
    for (mailbox, wanted) in wanted_by_mailbox(&job.messages, &Scope::All) {
        let located = locate(client, &mailbox, &wanted).await?;
        let uids: Vec<u32> = located.found.iter().map(|(_, uid)| *uid).collect();
        if on {
            client.add_flags(&uids, &[flag]).await?;
        } else {
            client.remove_flags(&uids, &[flag]).await?;
        }
        handled.extend(located.found.into_iter().map(|(message, _)| message));
    }
    require_any_handled(&job.messages, &handled)
}
