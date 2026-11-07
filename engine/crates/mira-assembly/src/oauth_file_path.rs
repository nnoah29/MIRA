//! Emplacement du fichier de configuration : `MIRA_CONFIG`, sinon `$XDG_CONFIG_HOME/mira/config.toml`, sinon `~/.config/mira/config.toml`.

use std::ffi::OsString;
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
    oauth_config_path_from_lookup(&|name| std::env::var_os(name))
}

/// Déduit le chemin par une fonction de recherche ; `APPDATA` remplace `XDG_CONFIG_HOME` absent (Windows)
pub fn oauth_config_path_from_lookup(lookup: &dyn Fn(&str) -> Option<OsString>) -> Option<PathBuf> {
    oauth_config_path(
        lookup("MIRA_CONFIG").map(PathBuf::from),
        lookup("XDG_CONFIG_HOME")
            .or_else(|| lookup("APPDATA"))
            .map(PathBuf::from),
        lookup("HOME").map(PathBuf::from),
    )
}
