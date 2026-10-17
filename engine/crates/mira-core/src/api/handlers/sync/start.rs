//! Lancement d'une synchronisation (tout, un compte, une boîte) : opération longue, demandes fusionnées (inv. 24).

use mira_vocabulary::{MiraError, OperationId};

use super::targets::{Target, resolve_targets};
use super::waiter::await_completions;
use crate::api::contract::{Response, SyncScope};
use crate::api::engine::Engine;
use crate::operations::run_in_background;
use crate::sync_service::Completion;

/// Met une demande dans la file de chaque compte visé et rend les attentes correspondantes
fn submit_all(
    engine: &Engine,
    targets: Vec<Target>,
    operation: &OperationId,
) -> Result<Vec<Completion>, MiraError> {
    let mut completions = Vec::new();
    for target in targets {
        let wait = engine
            .sync
            .submit(&target.account, target.scope, Some(operation.clone()))?;
        completions.push(wait);
    }
    Ok(completions)
}

/// Démarre une opération qui attend les passes demandées ; une demande refusée clôt l'opération en échec
pub(super) fn launch(
    engine: &Engine,
    kind: &str,
    targets: Vec<Target>,
) -> Result<Response, MiraError> {
    let handle = engine.operations.start(kind);
    let completions = match submit_all(engine, targets, handle.id()) {
        Ok(completions) => completions,
        Err(error) => {
            handle.complete(Err(&error));
            return Err(error);
        }
    };
    let info = engine.operations.get(handle.id())?;
    run_in_background(handle.clone(), await_completions(handle, completions));
    Ok(Response::Operation(info))
}

/// Demande la synchronisation, rend l'opération tout de suite ; la progression suit par événements
pub(crate) fn handle_start_sync(engine: &Engine, scope: SyncScope) -> Result<Response, MiraError> {
    let targets = resolve_targets(engine, &scope)?;
    launch(engine, "sync", targets)
}
