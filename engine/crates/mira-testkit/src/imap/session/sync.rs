//! Synchronisation de la vue du client avec l'état courant de la boîte.

use super::state::SessionState;
use super::sync_announce::announce_changes;
use crate::imap::response::reply::Reply;

/// Lignes non étiquetées annonçant EXISTS, EXPUNGE et changements de drapeaux
pub(crate) fn sync_selected(state: &mut SessionState) -> Reply {
    let mut reply = Reply::default();
    let (Some(user), Some(selected)) = (state.user.clone(), state.selected.clone()) else {
        return reply;
    };
    let view = state
        .ctx
        .store
        .read(|data| data.view(&user, &selected.name, selected.synced_modseq));
    match view {
        Some(view) if view.validity == selected.validity => {
            announce_changes(state, &view, &mut reply);
        }
        _ => {}
    }
    reply
}
