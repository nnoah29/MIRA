//! Domaine « comptes ».

mod add;
mod changes;
mod credential_login;
mod delete;
mod draft_plan;
mod list;
pub(crate) mod pause;
mod prepare;
mod register;
mod resume;
mod test_connection;
mod update;
mod verify_servers;

use mira_vocabulary::MiraError;

use crate::api::contract::{AccountRequest, Response};
use crate::api::engine::Engine;

/// Route une requête du domaine comptes
pub(crate) async fn route(engine: &Engine, request: AccountRequest) -> Result<Response, MiraError> {
    match request {
        AccountRequest::PrepareAccount { address, provider } => {
            prepare::handle_prepare_account(engine, &address, provider.as_ref())
        }
        AccountRequest::TestConnection { draft, credential } => {
            test_connection::handle_test_connection(engine, draft, credential).await
        }
        AccountRequest::AddAccount {
            draft,
            credential,
            idempotency_key,
        } => add::handle_add_account(engine, draft, credential, idempotency_key).await,
        AccountRequest::ListAccounts => list::handle_list_accounts(engine),
        AccountRequest::UpdateAccount { account, changes } => {
            update::handle_update_account(engine, &account, changes).await
        }
        AccountRequest::PauseAccount { account } => {
            pause::handle_pause_account(engine, &account).await
        }
        AccountRequest::ResumeAccount { account } => {
            resume::handle_resume_account(engine, &account)
        }
        AccountRequest::DeleteAccount { account } => {
            delete::handle_delete_account(engine, &account).await
        }
    }
}
