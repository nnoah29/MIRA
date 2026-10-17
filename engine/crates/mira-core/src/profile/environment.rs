//! Variables d'environnement qui décident de l'emplacement d'un profil, injectables en test.

use std::path::PathBuf;

/// Valeurs des variables `MIRA_HOME`, `XDG_DATA_HOME` et `HOME`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProfileEnvironment {
    pub mira_home: Option<PathBuf>,
    pub xdg_data_home: Option<PathBuf>,
    pub home: Option<PathBuf>,
}

impl ProfileEnvironment {
    /// Lit les variables du processus courant
    pub fn from_process() -> Self {
        Self {
            mira_home: std::env::var_os("MIRA_HOME").map(PathBuf::from),
            xdg_data_home: std::env::var_os("XDG_DATA_HOME").map(PathBuf::from),
            home: std::env::var_os("HOME").map(PathBuf::from),
        }
    }
}
