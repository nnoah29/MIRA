//! État du modèle local lu sur le disque et dans la mémoire : rien n'est créé, téléchargé ni lancé.

use mira_vocabulary::{AiProviderDescriptor, LocalModelStatus, LocalRuntimeStatus};

use super::extract_modes::find_executable;
use super::paths::LocalPaths;
use super::server_install::{current_platform, select_build};
use crate::api::Engine;

/// Fiche du fournisseur doté d'un runtime local, s'il est enregistré.
pub(crate) fn local_descriptor(engine: &Engine) -> Option<AiProviderDescriptor> {
    let all = engine.config.ai_registry.list();
    all.into_iter().find(|d| d.local_runtime.is_some())
}

/// Dit si le serveur est décrit pour la plateforme courante.
pub(crate) fn is_supported(descriptor: &AiProviderDescriptor) -> bool {
    let Some(runtime) = &descriptor.local_runtime else {
        return false;
    };
    current_platform().is_some_and(|p| select_build(&runtime.server, p).is_ok())
}

/// État de chaque modèle : installé, mémoire juste.
fn model_statuses(engine: &Engine, descriptor: &AiProviderDescriptor) -> Vec<LocalModelStatus> {
    let paths = LocalPaths::new(&engine.paths.root);
    let available = engine.config.resources.available_memory_mib();
    let models = descriptor
        .local_runtime
        .iter()
        .flat_map(|r| r.models.iter());
    models
        .map(|m| LocalModelStatus {
            id: m.id.clone(),
            display_name: m.display_name.clone(),
            tier: m.tier,
            size: m.size,
            license: m.license.clone(),
            supports_tools: m.supports_tools,
            ram_recommended_mib: m.ram_recommended_mib,
            installed: paths.model_file(m).is_file(),
            memory_warning: available.is_some_and(|a| a < u64::from(m.ram_recommended_mib)),
        })
        .collect()
}

/// Dit si le serveur est déjà extrait pour la plateforme courante.
fn server_installed(engine: &Engine, descriptor: &AiProviderDescriptor) -> bool {
    let paths = LocalPaths::new(&engine.paths.root);
    let runtime = descriptor.local_runtime.as_ref();
    let build = current_platform()
        .zip(runtime)
        .and_then(|(p, r)| select_build(&r.server, p).ok());
    build.is_some_and(|b| find_executable(&paths.server_dir(b), &b.executable).is_some())
}

/// Compose l'état complet ; sans fournisseur local enregistré, tout est vide et non pris en charge.
pub(crate) async fn local_status(engine: &Engine) -> LocalRuntimeStatus {
    let descriptor = local_descriptor(engine);
    let (models, installed) = match &descriptor {
        Some(d) => (model_statuses(engine, d), server_installed(engine, d)),
        None => (Vec::new(), false),
    };
    LocalRuntimeStatus {
        supported: descriptor.as_ref().is_some_and(is_supported),
        models,
        server_installed: installed,
        running_model: engine.ai.local_runtime.running_model().await,
        memory_available_mib: engine.config.resources.available_memory_mib(),
        installing: engine.ai.local_install.current(),
    }
}
