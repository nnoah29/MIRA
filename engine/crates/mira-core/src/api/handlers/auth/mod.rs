//! Domaine « authentification ».

mod begin_flow;
mod cancel_oauth;
mod complete_oauth;
mod reauthenticate;
mod start_oauth;
mod store_app_password;

use mira_vocabulary::MiraError;

use crate::api::contract::{AuthRequest, Response};
use crate::api::engine::Engine;

/// Route une requête du domaine authentification
pub(crate) async fn route(engine: &Engine, request: AuthRequest) -> Result<Response, MiraError> {
    match request {
        AuthRequest::StartOAuth { address, provider } => {
            start_oauth::handle_start_oauth(engine, &address, provider.as_ref()).await
        }
        AuthRequest::CompleteOAuth { flow } => {
            complete_oauth::handle_complete_oauth(engine, &flow).await
        }
        AuthRequest::CancelOAuth { flow } => cancel_oauth::handle_cancel_oauth(engine, &flow),
        AuthRequest::StoreAppPassword { account, password } => {
            store_app_password::handle_store_app_password(engine, &account, password)
        }
        AuthRequest::Reauthenticate { account } => {
            reauthenticate::handle_reauthenticate(engine, &account).await
        }
    }
}
