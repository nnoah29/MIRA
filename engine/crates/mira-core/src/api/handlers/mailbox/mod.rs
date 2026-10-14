//! Domaine « boîtes » : lecture de l'arbre et choix des boîtes synchronisées.

mod list;
mod set_synced;

use mira_vocabulary::MiraError;

use crate::api::contract::{MailboxRequest, Response};
use crate::api::engine::Engine;

/// Route une requête du domaine boîtes
pub(crate) async fn route(engine: &Engine, request: MailboxRequest) -> Result<Response, MiraError> {
    match request {
        MailboxRequest::ListMailboxes { account } => list::handle_list_mailboxes(engine, &account),
        MailboxRequest::SetSyncedMailboxes { account, mailboxes } => {
            set_synced::handle_set_synced_mailboxes(engine, &account, mailboxes)
        }
    }
}
