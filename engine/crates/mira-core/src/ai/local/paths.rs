//! Emplacements des fichiers du modèle local dans le dossier du profil : serveur, modèles, exécution.

use std::path::{Path, PathBuf};

use mira_vocabulary::{AiLocalModel, AiLocalServerBuild};

/// Dossier `local-ai` d'un profil et ses sous-dossiers.
#[derive(Debug, Clone)]
pub struct LocalPaths {
    root: PathBuf,
}

impl LocalPaths {
    /// Place le dossier `local-ai` sous la racine du profil
    pub fn new(profile_root: &Path) -> Self {
        Self {
            root: profile_root.join("local-ai"),
        }
    }

    /// Racine de tout ce que le modèle local occupe sur le disque
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Dossier d'une version du serveur, nommé d'après les 12 premiers caractères de la somme de son archive
    pub fn server_dir(&self, build: &AiLocalServerBuild) -> PathBuf {
        let short = build.sha256.get(..12).unwrap_or(&build.sha256);
        self.root.join("server").join(short)
    }

    /// Dossier des fichiers de modèle
    pub fn models_dir(&self) -> PathBuf {
        self.root.join("models")
    }

    /// Fichier d'un modèle installé
    pub fn model_file(&self, model: &AiLocalModel) -> PathBuf {
        self.models_dir().join(&model.file_name)
    }

    /// Dossier de la clé du lancement en cours
    pub fn run_dir(&self) -> PathBuf {
        self.root.join("run")
    }
}
