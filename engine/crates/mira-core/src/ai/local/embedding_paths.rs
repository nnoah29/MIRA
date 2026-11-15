//! Emplacement du fichier du modèle d'embeddings de la mémoire dans le dossier du profil.

use std::path::PathBuf;

use mira_vocabulary::AiLocalModel;

use super::paths::LocalPaths;

impl LocalPaths {
    /// Fichier du modèle d'embeddings de la mémoire, sous `embeddings/`
    pub fn embedding_file(&self, model: &AiLocalModel) -> PathBuf {
        self.root.join("embeddings").join(&model.file_name)
    }
}
