//! Adresse et clé du serveur local pour une discussion : le serveur est lancé ou réutilisé à la demande.

use mira_vocabulary::error::codes::incompatible_state;
use mira_vocabulary::{
    AiLocalModel, AiLocalRuntime, AiModel, AiModelList, AiProviderDescriptor, ErrorCategory,
    MiraError,
};

use super::extract_modes::find_executable;
use super::paths::LocalPaths;
use super::runtime::{LocalRuntimeHandle, RuntimeContext};
use super::server_install::{current_platform, select_build};
use crate::api::Engine;

/// Erreur : le modèle ou le serveur demandé n'est pas installé.
fn not_installed() -> MiraError {
    MiraError::new(
        ErrorCategory::IncompatibleState,
        incompatible_state::AI_MODEL_MISSING,
        "le modèle local n'est pas installé",
    )
}

/// Rend l'adresse et la clé du serveur de ce modèle, en le lançant si besoin.
pub(crate) async fn local_endpoint(
    engine: &Engine,
    descriptor: &AiProviderDescriptor,
    model_id: &str,
) -> Result<LocalRuntimeHandle, MiraError> {
    let runtime = descriptor
        .local_runtime
        .as_ref()
        .ok_or_else(not_installed)?;
    let model = find_model(runtime, model_id)?;
    let platform = current_platform().ok_or_else(unsupported_platform)?;
    let build = select_build(&runtime.server, platform)?;
    let paths = LocalPaths::new(&engine.paths.root);
    let server = paths.server_dir(build);
    let executable = find_executable(&server, &build.executable).ok_or_else(not_installed)?;
    let context = RuntimeContext {
        runtime,
        executable,
        run_dir: paths.run_dir(),
        models_dir: paths.models_dir(),
        resources: engine.config.resources.as_ref(),
    };
    engine
        .ai
        .local_runtime
        .ensure_running(&context, model)
        .await
}

/// Cherche le modèle dans la fiche du runtime, sinon « modèle manquant ».
fn find_model<'a>(runtime: &'a AiLocalRuntime, id: &str) -> Result<&'a AiLocalModel, MiraError> {
    runtime
        .models
        .iter()
        .find(|m| m.id == id)
        .ok_or_else(not_installed)
}

/// Erreur : la plateforme n'est pas prise en charge.
fn unsupported_platform() -> MiraError {
    MiraError::new(
        ErrorCategory::IncompatibleState,
        incompatible_state::AI_LOCAL_UNSUPPORTED_PLATFORM,
        "le modèle local n'est pas pris en charge sur cette plateforme",
    )
}

/// Modèles dont le fichier est présent sur le disque, lus sans lancer le serveur.
pub(crate) fn installed_models(engine: &Engine, descriptor: &AiProviderDescriptor) -> AiModelList {
    let paths = LocalPaths::new(&engine.paths.root);
    let models = descriptor
        .local_runtime
        .iter()
        .flat_map(|r| r.models.iter());
    AiModelList {
        models: models
            .filter(|m| paths.model_file(m).is_file())
            .map(|m| AiModel {
                id: m.id.clone(),
                created_at: None,
            })
            .collect(),
        fetched_at: engine.config.clock.now(),
    }
}

/// Identifiants des modèles dont le fichier est présent sur le disque.
pub(crate) fn installed_ids(engine: &Engine, descriptor: &AiProviderDescriptor) -> Vec<String> {
    installed_models(engine, descriptor)
        .models
        .into_iter()
        .map(|m| m.id)
        .collect()
}
