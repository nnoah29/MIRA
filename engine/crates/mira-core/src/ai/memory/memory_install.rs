//! Tâche d'installation de la mémoire : serveur puis modèle d'embeddings, progression sur une seule échelle, sans connexion de chat.

use std::sync::Arc;

use mira_vocabulary::error::codes::cancelled;
use mira_vocabulary::{AiLocalModel, AiLocalServerBuild, MiraError};

use super::embedding_install::install_embedding_model;
use crate::ai::local::{LocalInstallSlot, LocalPaths, ServerInstall, install_server};
use crate::operations::OperationHandle;
use crate::ports::SystemResources;

/// Tout ce que la tâche de fond emprunte au moteur, détenu pour pouvoir vivre seule.
pub(crate) struct MemoryInstallPlan {
    pub(crate) model: AiLocalModel,
    pub(crate) build: AiLocalServerBuild,
    pub(crate) paths: LocalPaths,
    pub(crate) resources: Arc<dyn SystemResources>,
    pub(crate) slot: LocalInstallSlot,
}

/// Exécute l'installation ; une annulation est un succès (l'opération passe alors à « annulée »).
pub(crate) async fn run_memory_install(
    plan: MemoryInstallPlan,
    handle: OperationHandle,
) -> Result<(), MiraError> {
    let outcome = install(&plan, &handle).await;
    plan.slot.end();
    match outcome {
        Err(error) if error.code == cancelled::BY_USER.0 => Ok(()),
        other => other,
    }
}

/// Serveur d'abord, puis fichier du modèle ; le serveur occupe le début de l'échelle commune.
async fn install(plan: &MemoryInstallPlan, handle: &OperationHandle) -> Result<(), MiraError> {
    let total = plan.build.size + plan.model.size;
    let root = plan.paths.server_dir(&plan.build);
    let server = ServerInstall {
        build: &plan.build,
        root: &root,
        resources: plan.resources.as_ref(),
    };
    let report = |done: u64, _: u64| handle.progress(done, total);
    install_server(&server, &report, &handle.cancel_flag()).await?;
    let offset = plan.build.size;
    let progress = |done: u64, _: u64| handle.progress(offset + done, total);
    install_embedding_model(
        &plan.paths,
        &plan.model,
        plan.resources.as_ref(),
        &progress,
        &handle.cancel_flag(),
    )
    .await
    .map(|_| ())
}
