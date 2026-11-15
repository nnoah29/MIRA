//! Emplacements de la mémoire de l'IA dans le profil, hors de `local-ai/` pour survivre au retrait d'un modèle de chat.

use std::path::PathBuf;

use mira_vocabulary::AiLocalModel;

use super::paths::LocalPaths;

impl LocalPaths {
    /// Dossier des modèles d'embeddings de la mémoire
    pub fn embedding_models_dir(&self) -> PathBuf {
        self.memory_root.join("embeddings")
    }

    /// Fichier du modèle d'embeddings de la mémoire
    pub fn embedding_file(&self, model: &AiLocalModel) -> PathBuf {
        self.embedding_models_dir().join(&model.file_name)
    }

    /// Dossier d'exécution du serveur d'embeddings, distinct de celui du chat pour ne jamais partager la clé
    pub fn embedding_run_dir(&self) -> PathBuf {
        self.memory_root.join("run")
    }
}
