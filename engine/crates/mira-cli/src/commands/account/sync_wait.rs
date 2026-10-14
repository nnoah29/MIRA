//! Attente de la première synchronisation d'un compte juste ajouté (`--sync-wait`).

use clap::Args;
use mira_assembly::api::{Engine, Request, Response, SyncRequest, SyncScope};
use mira_assembly::mira_vocabulary::MiraError;

use crate::commands::waiting::{WaitPolicy, execute_and_wait};

/// Option commune à `account add` et `account add-oauth`.
#[derive(Debug, Args)]
pub struct SyncWaitArgs {
    /// Attend la première synchronisation du compte avant de rendre la main.
    #[arg(long)]
    pub sync_wait: bool,
    /// Durée maximale de chaque attente (redirection OAuth, synchronisation), en secondes.
    #[arg(long, default_value_t = 300)]
    pub timeout: u64,
}

/// Lance la synchronisation du compte ajouté et l'attend ; sans l'option, rend l'ajout tel quel.
pub async fn settle_first_sync(
    engine: &Engine,
    added: Response,
    wait: &SyncWaitArgs,
) -> Result<Response, MiraError> {
    let Response::Account(account) = &added else {
        return Ok(added);
    };
    if !wait.sync_wait {
        return Ok(added);
    }
    let scope = SyncScope::Account {
        account: account.id.clone(),
    };
    let request = Request::Sync(SyncRequest::StartSync { scope });
    let policy = WaitPolicy::Operation {
        seconds: wait.timeout,
    };
    execute_and_wait(engine, request, policy).await?;
    Ok(added)
}
