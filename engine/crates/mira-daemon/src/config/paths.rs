//! Chemins par défaut du démon : socket dans le dossier d'exécution, jeton sous le profil.

use std::path::{Path, PathBuf};

use mira_assembly::mira_vocabulary::{MiraError, ProfileId};
use mira_assembly::profile::{ProfileEnvironment, ProfilePaths};

use super::DaemonConfig;

/// Emplacements de la socket Unix et du fichier du jeton.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DaemonPaths {
    pub socket: PathBuf,
    pub token: PathBuf,
}

/// Environnement du processus, avec le dossier de données de l'option `--home` s'il est donné.
pub fn environment_of(config: &DaemonConfig) -> ProfileEnvironment {
    let mut environment = ProfileEnvironment::from_process();
    if let Some(home) = &config.home {
        environment.mira_home = Some(home.clone());
    }
    environment
}

/// Résout la socket et le jeton : l'option l'emporte, sinon `<exécution>/mira/<profil>.sock` ou le dossier du profil.
pub fn resolve_paths(
    config: &DaemonConfig,
    environment: &ProfileEnvironment,
    runtime_dir: Option<&Path>,
) -> Result<DaemonPaths, MiraError> {
    let root = ProfilePaths::resolve(&ProfileId::from_raw(&config.profile), environment)?.root;
    let socket = match (&config.socket, runtime_dir) {
        (Some(explicit), _) => explicit.clone(),
        (None, Some(runtime)) => runtime.join(format!("mira/{}.sock", config.profile)),
        (None, None) => root.join("daemon.sock"),
    };
    let token = config
        .token_file
        .clone()
        .unwrap_or_else(|| root.join("daemon.token"));
    Ok(DaemonPaths { socket, token })
}
