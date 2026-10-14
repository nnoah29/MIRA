//! Domaine « file d'actions » : lister, relancer, annuler.

mod cancel;
mod list;
mod retry;

use mira_vocabulary::MiraError;

use crate::api::contract::{QueueRequest, Response};
use crate::api::engine::Engine;

/// Route une requête du domaine file d'actions
pub(crate) async fn route(engine: &Engine, request: QueueRequest) -> Result<Response, MiraError> {
    match request {
        QueueRequest::ListActions { account, states } => {
            list::handle_list_actions(engine, &account, &states)
        }
        QueueRequest::RetryAction { action } => retry::handle_retry_action(engine, &action),
        QueueRequest::CancelAction { action } => cancel::handle_cancel_action(engine, &action),
    }
}
