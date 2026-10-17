//! Écriture des annonces de synchronisation.

use super::state::SessionState;
use crate::imap::parser::uidset_format::format_uid_set;
use crate::imap::response::fetch_flags::flags_update;
use crate::imap::response::reply::Reply;
use crate::imap::store::view::MailboxView;

/// Annonce la disparition de messages et met à jour la liste connue du client
pub(crate) fn announce_removed(
    reply: &mut Reply,
    known: &mut Vec<u32>,
    removed: &[u32],
    qresync: bool,
) {
    if qresync && !removed.is_empty() {
        reply.untagged(&format!("VANISHED {}", format_uid_set(removed)));
        known.retain(|uid| !removed.contains(uid));
        return;
    }
    for uid in removed {
        if let Some(index) = known.iter().position(|known_uid| known_uid == uid) {
            known.remove(index);
            reply.untagged(&format!("{} EXPUNGE", index + 1));
        }
    }
}

/// Annonce suppressions, arrivées et changements de drapeaux puis mémorise la vue
pub(crate) fn announce_changes(state: &mut SessionState, view: &MailboxView, reply: &mut Reply) {
    let (condstore, qresync) = (state.enabled.condstore, state.enabled.qresync);
    let Some(selected) = state.selected.as_mut() else {
        return;
    };
    let removed: Vec<u32> = selected
        .uids
        .iter()
        .copied()
        .filter(|uid| !view.uids.contains(uid))
        .collect();
    announce_removed(reply, &mut selected.uids, &removed, qresync);
    if view.uids.len() != selected.uids.len() {
        reply.untagged(&format!("{} EXISTS", view.uids.len()));
    }
    for row in view
        .changed
        .iter()
        .filter(|row| selected.uids.contains(&row.uid))
    {
        reply.untagged(&flags_update(row, condstore));
    }
    selected.uids = view.uids.clone();
    selected.synced_modseq = view.highest_modseq;
}
