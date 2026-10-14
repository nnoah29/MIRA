//! Libellés du service (`X-GM-LABELS`) : ajout, retrait, et archivage par retrait de la réception.

use std::collections::BTreeSet;

use mira_vocabulary::{MailboxRole, MiraError};

use super::locate::{Scope, locate, wanted_by_mailbox};
use super::locate_require::require_any_handled;
use crate::actions::replay::job::{NamedMailbox, ReplayJob};
use crate::imap::ImapClient;

/// Nom du libellé : `\Inbox` pour la réception, le nom de la boîte sinon
fn label_name(mailbox: &NamedMailbox) -> String {
    match mailbox.role {
        MailboxRole::Inbox => "\\Inbox".to_string(),
        _ => mailbox.name.clone(),
    }
}

/// Ajoute le libellé sur chaque message retrouvé ; échec définitif si aucun n'existe plus
pub(super) async fn add_label(
    client: &mut ImapClient,
    job: &ReplayJob,
    mailbox: &NamedMailbox,
) -> Result<(), MiraError> {
    let labels = [label_name(mailbox)];
    let mut handled = BTreeSet::new();
    for (name, wanted) in wanted_by_mailbox(&job.messages, &Scope::All) {
        let located = locate(client, &name, &wanted).await?;
        let uids: Vec<u32> = located.found.iter().map(|(_, uid)| *uid).collect();
        client.add_labels(&uids, &labels).await?;
        handled.extend(located.found.into_iter().map(|(message, _)| message));
    }
    require_any_handled(&job.messages, &handled)
}

/// Retire le libellé là où le message apparaît dans cette boîte ; déjà retiré = objectif atteint
pub(super) async fn remove_label(
    client: &mut ImapClient,
    job: &ReplayJob,
    mailbox: &NamedMailbox,
) -> Result<(), MiraError> {
    let labels = [label_name(mailbox)];
    let scope = Scope::Only(mailbox.name.clone());
    for (name, wanted) in wanted_by_mailbox(&job.messages, &scope) {
        let located = locate(client, &name, &wanted).await?;
        let uids: Vec<u32> = located.found.iter().map(|(_, uid)| *uid).collect();
        client.remove_labels(&uids, &labels).await?;
    }
    Ok(())
}
