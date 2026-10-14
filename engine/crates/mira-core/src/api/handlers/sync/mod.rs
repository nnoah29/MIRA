//! Domaine « synchronisation » : lancer, mettre en pause, état, remonter la plage, resynchroniser une boîte.

mod extend_window;
mod pause;
mod resync_mailbox;
mod start;
mod status;
mod status_view;
mod targets;
mod waiter;

use mira_vocabulary::MiraError;

use crate::api::contract::{Response, SyncRequest};
use crate::api::engine::Engine;

/// Route une requête du domaine synchronisation
pub(crate) async fn route(engine: &Engine, request: SyncRequest) -> Result<Response, MiraError> {
    match request {
        SyncRequest::StartSync { scope } => start::handle_start_sync(engine, scope),
        SyncRequest::PauseSync { account } => pause::handle_pause_sync(engine, &account).await,
        SyncRequest::GetSyncStatus => status::handle_get_sync_status(engine),
        SyncRequest::ExtendWindow { mailbox, days } => {
            extend_window::handle_extend_window(engine, mailbox, days)
        }
        SyncRequest::ResyncMailbox { mailbox } => {
            resync_mailbox::handle_resync_mailbox(engine, mailbox)
        }
    }
}
