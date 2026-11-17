//! Installation du modèle d'embeddings de la mémoire : validée tout de suite, exécutée en arrière-plan, annulable.

use mira_vocabulary::{LocalInstallInfo, MiraError};

use super::local_install::busy;
use crate::ai::local::embedding_install_target;
use crate::ai::memory::{MemoryInstallPlan, run_memory_install};
use crate::api::contract::Response;
use crate::api::engine::Engine;
use crate::operations::run_in_background;

/// Valide la demande, réserve l'emplacement partagé avec le chat et lance l'installation ; rend la fiche de l'opération
pub(crate) fn handle_install_memory_model(engine: &Engine) -> Result<Response, MiraError> {
    let (model, build, paths) = embedding_install_target(engine)?;
    let plan = MemoryInstallPlan {
        model,
        build,
        paths,
        resources: engine.config.resources.clone(),
        slot: engine.ai.local_install.clone(),
    };
    let handle = engine.operations.start("ai_memory_install");
    let info = LocalInstallInfo {
        model_id: plan.model.id.clone(),
        operation: handle.id().clone(),
    };
    if !plan.slot.try_begin(info) {
        handle.complete(Ok(()));
        return Err(busy());
    }
    let operation = engine.operations.get(handle.id())?;
    run_in_background(handle.clone(), run_memory_install(plan, handle));
    Ok(Response::Operation(operation))
}
