//! Retrait d'un modèle local : arrêt du serveur, fichiers supprimés, connexion corrigée ou retirée.

use mira_vocabulary::error::codes::incompatible_state;
use mira_vocabulary::{ErrorCategory, MiraError};

use super::connection_view::done;
use crate::ai::local::{LocalPaths, installed_ids, local_descriptor, trim_local_connection};
use crate::api::contract::Response;
use crate::api::engine::Engine;

/// Supprime le modèle ; quand il n'en reste aucun, tout le dossier du modèle local et sa connexion disparaissent
pub(crate) async fn handle_remove_local_model(
    engine: &Engine,
    model_id: &str,
) -> Result<Response, MiraError> {
    let descriptor = local_descriptor(engine).ok_or_else(unsupported)?;
    let model = descriptor
        .local_runtime
        .iter()
        .flat_map(|r| r.models.iter())
        .find(|m| m.id == model_id)
        .ok_or_else(unsupported)?;
    stop_if_serving(engine, model_id).await?;
    let paths = LocalPaths::new(&engine.paths.root);
    delete_model_files(&paths.model_file(model)).await;
    let remaining = installed_ids(engine, &descriptor);
    if remaining.is_empty() {
        let _ = tokio::fs::remove_dir_all(paths.root()).await;
    }
    let clock = engine.config.clock.as_ref();
    trim_local_connection(&engine.database, clock, &descriptor, model_id, &remaining)?;
    Ok(done())
}

/// Refuse tout retrait pendant une installation, et n'arrête le serveur que s'il sert ce modèle.
async fn stop_if_serving(engine: &Engine, model_id: &str) -> Result<(), MiraError> {
    if engine.ai.local_install.current().is_some() {
        return Err(busy());
    }
    if engine.ai.local_runtime.running_model().await.as_deref() == Some(model_id) {
        engine.ai.local_runtime.stop().await;
    }
    Ok(())
}

/// Supprime le fichier du modèle et son éventuel fichier partiel ; l'absence n'est pas une erreur.
async fn delete_model_files(file: &std::path::Path) {
    let _ = tokio::fs::remove_file(file).await;
    let mut part = file.as_os_str().to_os_string();
    part.push(".part");
    let _ = tokio::fs::remove_file(part).await;
}

/// Erreur : modèle local inconnu ou non pris en charge.
fn unsupported() -> MiraError {
    MiraError::new(
        ErrorCategory::IncompatibleState,
        incompatible_state::AI_LOCAL_UNSUPPORTED_PLATFORM,
        "modèle local inconnu ou non pris en charge",
    )
}

/// Erreur : une installation est en cours, on ne retire rien pendant ce temps.
fn busy() -> MiraError {
    MiraError::new(
        ErrorCategory::IncompatibleState,
        incompatible_state::AI_LOCAL_BUSY,
        "une installation du modèle local est en cours",
    )
}
