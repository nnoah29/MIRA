//! Mise en place du serveur d'embeddings de la mémoire : fiche, exécutable, dossiers propres et modèle installé.

use mira_vocabulary::error::codes::incompatible_state;
use mira_vocabulary::{
    AiLocalModel, AiLocalServerBuild, AiProviderDescriptor, ErrorCategory, MiraError,
};

use super::extract_modes::find_executable;
use super::paths::LocalPaths;
use super::server_install::{current_platform, select_build};
use super::status::local_descriptor;
use crate::ai::memory::{LocalEmbedderSetup, embedding_runtime, is_embedding_installed};
use crate::api::Engine;

/// Erreur : le serveur ou le modèle d'embeddings n'est pas installé.
fn missing() -> MiraError {
    MiraError::new(
        ErrorCategory::IncompatibleState,
        incompatible_state::AI_MODEL_MISSING,
        "le modèle d'embeddings n'est pas installé",
    )
}

/// Erreur : la plateforme ou la fiche ne prévoit pas d'embeddings locaux.
fn unsupported() -> MiraError {
    MiraError::new(
        ErrorCategory::IncompatibleState,
        incompatible_state::AI_LOCAL_UNSUPPORTED_PLATFORM,
        "les embeddings locaux ne sont pas pris en charge sur cette plateforme",
    )
}

/// Rassemble de quoi lancer le serveur d'embeddings ; refuse plateforme inconnue, serveur ou modèle absent.
pub(crate) fn embedding_setup(
    engine: &Engine,
    descriptor: &AiProviderDescriptor,
) -> Result<LocalEmbedderSetup, MiraError> {
    let runtime = descriptor.local_runtime.as_ref().ok_or_else(unsupported)?;
    let embedding = runtime.embedding.as_ref().ok_or_else(unsupported)?;
    let platform = current_platform().ok_or_else(unsupported)?;
    let build = select_build(&runtime.server, platform)?;
    let paths = LocalPaths::new(&engine.paths.root);
    if !is_embedding_installed(&paths, &embedding.model) {
        return Err(missing());
    }
    let executable =
        find_executable(&paths.server_dir(build), &build.executable).ok_or_else(missing)?;
    Ok(LocalEmbedderSetup {
        manager: engine.ai.memory.manager(),
        runtime: embedding_runtime(runtime, &embedding.server_args),
        executable,
        run_dir: paths.embedding_run_dir(),
        models_dir: paths.embedding_models_dir(),
        resources: engine.config.resources.clone(),
        model: embedding.model.clone(),
    })
}

/// Ce que l'installation de la mémoire télécharge : modèle d'embeddings, serveur de la plateforme et emplacements du profil.
pub(crate) fn embedding_install_target(
    engine: &Engine,
) -> Result<(AiLocalModel, AiLocalServerBuild, LocalPaths), MiraError> {
    let descriptor = local_descriptor(engine).ok_or_else(unsupported)?;
    let runtime = descriptor.local_runtime.as_ref().ok_or_else(unsupported)?;
    let embedding = runtime.embedding.as_ref().ok_or_else(unsupported)?;
    let platform = current_platform().ok_or_else(unsupported)?;
    let build = select_build(&runtime.server, platform).map_err(|_| unsupported())?;
    let paths = LocalPaths::new(&engine.paths.root);
    Ok((embedding.model.clone(), build.clone(), paths))
}
