//! Tâche d'installation : serveur puis modèle, progression sur une seule échelle, connexion créée à la fin.

use std::sync::Arc;

use mira_vocabulary::error::codes::cancelled;
use mira_vocabulary::{AiLocalModel, AiLocalServerBuild, AiProviderDescriptor, MiraError};

use super::connection::ensure_local_connection;
use super::download::{DownloadJob, download_file};
use super::install_slot::LocalInstallSlot;
use super::paths::LocalPaths;
use super::server_install::{ServerInstall, install_server};
use crate::operations::OperationHandle;
use crate::ports::{Clock, IdGenerator, SystemResources};
use crate::storage::Database;

/// Tout ce que la tâche de fond emprunte au moteur, détenu pour pouvoir vivre seule.
pub(crate) struct InstallPlan {
    pub(crate) descriptor: AiProviderDescriptor,
    pub(crate) model: AiLocalModel,
    pub(crate) build: AiLocalServerBuild,
    pub(crate) paths: LocalPaths,
    pub(crate) resources: Arc<dyn SystemResources>,
    pub(crate) database: Arc<Database>,
    pub(crate) ids: Arc<dyn IdGenerator>,
    pub(crate) clock: Arc<dyn Clock>,
    pub(crate) slot: LocalInstallSlot,
}

/// Exécute l'installation ; une annulation est un succès (l'opération passe alors à « annulée »).
pub(crate) async fn run_install(
    plan: InstallPlan,
    handle: OperationHandle,
) -> Result<(), MiraError> {
    let outcome = install(&plan, &handle).await;
    plan.slot.end();
    match outcome {
        Err(error) if error.code == cancelled::BY_USER.0 => Ok(()),
        other => other,
    }
}

/// Serveur, puis fichier du modèle, puis connexion.
async fn install(plan: &InstallPlan, handle: &OperationHandle) -> Result<(), MiraError> {
    let total = plan.build.size + plan.model.size;
    install_the_server(plan, handle, total).await?;
    let cancel = handle.cancel_flag();
    let dest = plan.paths.model_file(&plan.model);
    let download = DownloadJob {
        url: &plan.model.url,
        sha256: &plan.model.sha256,
        size: plan.model.size,
        dest: &dest,
        resources: plan.resources.as_ref(),
    };
    let offset = plan.build.size;
    download_file(
        &download,
        &|done, _| handle.progress(offset + done, total),
        &cancel,
    )
    .await?;
    let (ids, clock) = (plan.ids.as_ref(), plan.clock.as_ref());
    ensure_local_connection(&plan.database, ids, clock, &plan.descriptor, &plan.model.id)
}

/// Installe le serveur ; sa progression occupe le début de l'échelle commune.
async fn install_the_server(
    plan: &InstallPlan,
    handle: &OperationHandle,
    total: u64,
) -> Result<(), MiraError> {
    let root = plan.paths.server_dir(&plan.build);
    let server = ServerInstall {
        build: &plan.build,
        root: &root,
        resources: plan.resources.as_ref(),
    };
    let report = |done: u64, _: u64| handle.progress(done, total);
    install_server(&server, &report, &handle.cancel_flag())
        .await
        .map(|_| ())
}
