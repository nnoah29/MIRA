//! Attente de la fin des tâches lancées par une commande : le CLI vit le temps d'une commande.
//! Une opération longue est toujours attendue ; une action de la file l'est avec `--wait`.

pub mod actions;
pub mod failure;
pub mod operations;
pub mod policy;

use clap::Args;
use mira_assembly::api::{Engine, Request, Response};
use mira_assembly::mira_vocabulary::MiraError;

/// Ce que le CLI attend avant de rendre la main.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WaitPolicy {
    /// Rend la réponse du moteur telle quelle.
    Immediate,
    /// Attend la fin de l'opération longue lancée, au plus `seconds` secondes.
    Operation { seconds: u64 },
    /// Attend que les actions créées quittent la file, au plus `seconds` secondes.
    Actions { seconds: u64 },
}

/// Attente facultative des actions créées par une écriture.
#[derive(Debug, Args)]
pub struct WaitArgs {
    /// Attend que l'action soit rejouée vers le serveur avant de rendre la main.
    #[arg(long)]
    pub wait: bool,
    /// Durée maximale de l'attente, en secondes.
    #[arg(long, default_value_t = 60)]
    pub timeout: u64,
}

/// Délai maximal de l'attente d'une opération longue.
#[derive(Debug, Args)]
pub struct OperationWaitArgs {
    /// Durée maximale de l'attente de l'opération, en secondes.
    #[arg(long, default_value_t = 120)]
    pub timeout: u64,
}

impl WaitArgs {
    /// Politique d'attente demandée par les options.
    pub fn policy(&self) -> WaitPolicy {
        if self.wait {
            WaitPolicy::Actions {
                seconds: self.timeout,
            }
        } else {
            WaitPolicy::Immediate
        }
    }
}

impl OperationWaitArgs {
    /// Politique d'attente d'une opération longue.
    pub fn policy(&self) -> WaitPolicy {
        WaitPolicy::Operation {
            seconds: self.timeout,
        }
    }
}

/// Exécute la requête puis attend, selon la politique, ce qu'elle a lancé.
pub async fn execute_and_wait(
    engine: &Engine,
    request: Request,
    policy: WaitPolicy,
) -> Result<Response, MiraError> {
    let response = engine.execute(request.clone()).await?;
    match (policy, response) {
        (WaitPolicy::Operation { seconds }, Response::Operation(info)) => {
            operations::finish_operation(engine, info, seconds).await
        }
        (WaitPolicy::Actions { seconds }, response) => {
            let ids = actions::awaited_ids(&request, &response);
            actions::settle_actions(engine, ids, response, seconds).await
        }
        (_, other) => Ok(other),
    }
}
