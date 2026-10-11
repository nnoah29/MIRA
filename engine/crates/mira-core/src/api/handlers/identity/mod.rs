//! Domaine « identités d'envoi ».

mod add;
mod list;
mod set_default;
mod update;

use mira_vocabulary::MiraError;

use crate::api::contract::{IdentityRequest, Response};
use crate::api::engine::Engine;

/// Route une requête du domaine identités
pub(crate) async fn route(
    engine: &Engine,
    request: IdentityRequest,
) -> Result<Response, MiraError> {
    match request {
        IdentityRequest::ListIdentities { account } => {
            list::handle_list_identities(engine, &account)
        }
        IdentityRequest::AddIdentity(new) => add::handle_add_identity(engine, new),
        IdentityRequest::UpdateIdentity {
            identity,
            display_name,
            signature,
        } => update::handle_update_identity(engine, &identity, display_name, signature),
        IdentityRequest::SetDefaultIdentity { identity } => {
            set_default::handle_set_default_identity(engine, &identity)
        }
    }
}
