//! Variables d'environnement qui décident de l'emplacement d'un profil, injectables en test.

use std::ffi::OsString;
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
        Self::from_lookup(&|name| std::env::var_os(name))
    }

    /// Lit les variables par une fonction de recherche ; `LOCALAPPDATA` remplace `XDG_DATA_HOME` absent
    pub fn from_lookup(lookup: &dyn Fn(&str) -> Option<OsString>) -> Self {
        Self {
            mira_home: lookup("MIRA_HOME").map(PathBuf::from),
            xdg_data_home: lookup("XDG_DATA_HOME")
                .or_else(|| lookup("LOCALAPPDATA"))
                .map(PathBuf::from),
            home: lookup("HOME").map(PathBuf::from),
        }
    }
}
