//! Installation d'un modèle local : validée tout de suite, exécutée en arrière-plan, annulable.

use mira_vocabulary::error::codes::{incompatible_state, not_found};
use mira_vocabulary::{AiLocalModel, ErrorCategory, LocalInstallInfo, MiraError};

use crate::ai::local::{
    InstallPlan, LocalPaths, current_platform, local_descriptor, run_install, select_build,
};
use crate::api::contract::Response;
use crate::api::engine::Engine;
use crate::operations::run_in_background;

/// Valide la demande, réserve l'emplacement et lance l'installation ; rend la fiche de l'opération
pub(crate) fn handle_install_local_model(
    engine: &Engine,
    model_id: &str,
) -> Result<Response, MiraError> {
    let plan = build_plan(engine, model_id)?;
    let handle = engine.operations.start("ai_local_install");
    let info = LocalInstallInfo {
        model_id: model_id.to_string(),
        operation: handle.id().clone(),
    };
    if !plan.slot.try_begin(info) {
        handle.complete(Ok(()));
        return Err(busy());
    }
    let operation = engine.operations.get(handle.id())?;
    run_in_background(handle.clone(), run_install(plan, handle));
    Ok(Response::Operation(operation))
}

/// Erreur : une installation est déjà en cours.
fn busy() -> MiraError {
    MiraError::new(
        ErrorCategory::IncompatibleState,
        incompatible_state::AI_LOCAL_BUSY,
        "une installation du modèle local est déjà en cours",
    )
}

/// Rassemble ce que la tâche emprunte au moteur ; refuse plateforme ou modèle inconnus.
fn build_plan(engine: &Engine, model_id: &str) -> Result<InstallPlan, MiraError> {
    let unsupported = || {
        MiraError::new(
            ErrorCategory::IncompatibleState,
            incompatible_state::AI_LOCAL_UNSUPPORTED_PLATFORM,
            "le modèle local n'est pas pris en charge sur cette plateforme",
        )
    };
    let descriptor = local_descriptor(engine).ok_or_else(unsupported)?;
    let runtime = descriptor.local_runtime.clone().ok_or_else(unsupported)?;
    let platform = current_platform().ok_or_else(unsupported)?;
    let build = select_build(&runtime.server, platform)
        .map_err(|_| unsupported())?
        .clone();
    let model = find_model(&runtime.models, model_id)?;
    Ok(InstallPlan {
        descriptor,
        model,
        build,
        paths: LocalPaths::new(&engine.paths.root),
        resources: engine.config.resources.clone(),
        database: engine.database.clone(),
        ids: engine.config.ids.clone(),
        clock: engine.config.clock.clone(),
        slot: engine.ai.local_install.clone(),
    })
}

/// Cherche le modèle demandé, sinon `not_found.ai_provider`-like : modèle inconnu.
fn find_model(models: &[AiLocalModel], id: &str) -> Result<AiLocalModel, MiraError> {
    models.iter().find(|m| m.id == id).cloned().ok_or_else(|| {
        MiraError::new(
            ErrorCategory::NotFound,
            not_found::AI_PROVIDER,
            "modèle local inconnu",
        )
    })
}
