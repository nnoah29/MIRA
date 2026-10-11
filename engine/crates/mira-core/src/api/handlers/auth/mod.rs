//! Domaine « authentification ».

mod store_app_password;

use mira_vocabulary::MiraError;

use crate::api::contract::{AuthRequest, Response};
use crate::api::engine::Engine;

/// Route une requête du domaine authentification
pub(crate) async fn route(engine: &Engine, request: AuthRequest) -> Result<Response, MiraError> {
    match request {
        AuthRequest::StoreAppPassword { account, password } => {
            store_app_password::handle_store_app_password(engine, &account, password)
        }
    }
}
