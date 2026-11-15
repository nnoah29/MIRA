//! Installation du modèle d'embeddings de la mémoire : téléchargement épinglé, vérifié par taille et SHA-256.

use std::path::PathBuf;

use mira_vocabulary::{AiLocalModel, MiraError};

use crate::ai::local::{CancelFlag, DownloadJob, LocalPaths, download_file};
use crate::ports::SystemResources;

/// Télécharge le modèle sous `local-ai/embeddings/` et rend son chemin ; sans effet s'il est déjà installé
pub async fn install_embedding_model(
    paths: &LocalPaths,
    model: &AiLocalModel,
    resources: &dyn SystemResources,
    progress: &(dyn Fn(u64, u64) + Send + Sync),
    cancel: &CancelFlag,
) -> Result<PathBuf, MiraError> {
    let dest = paths.embedding_file(model);
    let job = DownloadJob {
        url: &model.url,
        sha256: &model.sha256,
        size: model.size,
        dest: &dest,
        resources,
    };
    download_file(&job, progress, cancel).await?;
    Ok(dest)
}

/// Dit si le fichier du modèle est déjà installé à la bonne taille
pub fn is_embedding_installed(paths: &LocalPaths, model: &AiLocalModel) -> bool {
    std::fs::metadata(paths.embedding_file(model))
        .is_ok_and(|m| m.is_file() && m.len() == model.size)
}
