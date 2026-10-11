//! Test de connexion d'un compte avant son ajout (S3) : rien n'est enregistré.

use mira_vocabulary::MiraError;

use super::credential_login::test_credential;
use super::draft_plan::plan_draft;
use crate::accounts::check_imap_login;
use crate::api::contract::{AccountDraft, CredentialInput, Response};
use crate::api::engine::Engine;

/// Se connecte au serveur IMAP avec le secret fourni, sans rien écrire ni ranger
pub(crate) async fn handle_test_connection(
    engine: &Engine,
    draft: AccountDraft,
    credential: CredentialInput,
) -> Result<Response, MiraError> {
    let plan = plan_draft(engine, &draft, &credential)?;
    let login = test_credential(engine, &plan, credential)?;
    check_imap_login(&plan.servers.connect, &login).await?;
    Ok(Response::Done)
}
