//! Emplacement du fichier de configuration : `MIRA_CONFIG`, sinon `$XDG_CONFIG_HOME/mira/config.toml`, sinon `~/.config/mira/config.toml`.

use std::path::PathBuf;

const CONFIG_DIRECTORY: &str = "mira";
const CONFIG_FILE: &str = "config.toml";

/// Choisit le chemin d'après les trois variables d'environnement, dans l'ordre de priorité
pub fn oauth_config_path(
    explicit: Option<PathBuf>,
    xdg_config_home: Option<PathBuf>,
    home: Option<PathBuf>,
) -> Option<PathBuf> {
    if let Some(path) = explicit {
        return Some(path);
    }
    let base = xdg_config_home
        .filter(|path| path.is_absolute())
        .or_else(|| home.map(|path| path.join(".config")))?;
    Some(base.join(CONFIG_DIRECTORY).join(CONFIG_FILE))
}

/// Lit les variables du processus et en déduit le chemin du fichier
pub fn oauth_config_path_from_process() -> Option<PathBuf> {
    oauth_config_path(
        std::env::var_os("MIRA_CONFIG").map(PathBuf::from),
        std::env::var_os("XDG_CONFIG_HOME").map(PathBuf::from),
        std::env::var_os("HOME").map(PathBuf::from),
    )
}
