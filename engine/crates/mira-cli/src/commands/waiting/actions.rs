//! Attente des actions de la file : jusqu'à ce qu'elles ne soient plus ni en attente ni en cours.

use std::time::Duration;

use mira_assembly::api::{
    ComposeRequest, Engine, EngineRequest, QueueRequest, Request, Response, WriteResponse,
};
use mira_assembly::mira_vocabulary::{AccountId, ActionId, ActionState, MiraError, PendingAction};
use tokio::time::{Instant, sleep};

use super::failure::{broken_action_error, timeout_error};

const POLL_INTERVAL: Duration = Duration::from_millis(50);

/// Actions à attendre : celles créées par la réponse, ou l'action relancée ou résolue par la requête.
pub fn awaited_ids(request: &Request, response: &Response) -> Vec<ActionId> {
    match (request, response) {
        (_, Response::Write(WriteResponse::Action(id))) => vec![id.clone()],
        (_, Response::Write(WriteResponse::Actions(ids))) => ids.clone(),
        (Request::Queue(QueueRequest::RetryAction { action }), _)
        | (Request::Compose(ComposeRequest::ResolveUncertainSend { action, .. }), _) => {
            vec![action.clone()]
        }
        _ => Vec::new(),
    }
}

/// Actions de la file d'un compte, tous états confondus.
async fn account_actions(
    engine: &Engine,
    account: &AccountId,
) -> Result<Vec<PendingAction>, MiraError> {
    let request = QueueRequest::ListActions {
        account: account.clone(),
        states: Vec::new(),
    };
    match engine.execute(Request::Queue(request)).await? {
        Response::Write(WriteResponse::QueuedActions(list)) => Ok(list),
        _ => Ok(Vec::new()),
    }
}

/// Parmi les actions de tous les comptes, celles dont l'identifiant est demandé.
async fn queued_among(engine: &Engine, ids: &[ActionId]) -> Result<Vec<PendingAction>, MiraError> {
    let Response::Status(status) = engine
        .execute(Request::Engine(EngineRequest::GetStatus))
        .await?
    else {
        return Ok(Vec::new());
    };
    let mut found = Vec::new();
    for entry in &status.accounts {
        let actions = account_actions(engine, &entry.account.id).await?;
        found.extend(
            actions
                .into_iter()
                .filter(|action| ids.contains(&action.id)),
        );
    }
    Ok(found)
}

/// Dit si l'action attend encore son tour ou est en cours de rejeu.
fn is_unsettled(action: &PendingAction) -> bool {
    matches!(action.state, ActionState::Pending | ActionState::InProgress)
}

/// Attend que les actions de la réponse soient rejouées ; un échec ou un doute devient une erreur.
pub async fn settle_actions(
    engine: &Engine,
    ids: Vec<ActionId>,
    response: Response,
    seconds: u64,
) -> Result<Response, MiraError> {
    let deadline = Instant::now() + Duration::from_secs(seconds);
    loop {
        let queued = queued_among(engine, &ids).await?;
        if !queued.iter().any(is_unsettled) {
            return match queued
                .iter()
                .find(|action| action.state != ActionState::Succeeded)
            {
                Some(broken) if broken.state != ActionState::Cancelled => {
                    Err(broken_action_error(broken))
                }
                _ => Ok(response),
            };
        }
        if Instant::now() >= deadline {
            return Err(timeout_error(seconds));
        }
        sleep(POLL_INTERVAL).await;
    }
}
