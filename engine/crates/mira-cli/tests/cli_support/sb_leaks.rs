//! Recherche d'un secret dans les sorties du CLI et dans tous les fichiers du profil (base comprise).

use std::fs;
use std::path::Path;

use super::CliResult;
use super::sb_world::SbWorld;

/// Parcourt un dossier et dit si un fichier contient ces octets.
fn directory_contains(folder: &Path, needle: &[u8]) -> bool {
    let entries = fs::read_dir(folder).unwrap();
    entries.map(|entry| entry.unwrap().path()).any(|path| {
        if path.is_dir() {
            return directory_contains(&path, needle);
        }
        let bytes = fs::read(&path).unwrap();
        bytes.windows(needle.len()).any(|window| window == needle)
    })
}

/// Exige que le secret ne figure ni sur la sortie standard ni sur la sortie d'erreur.
pub fn assert_outputs_clean(results: &[&CliResult], secret: &str) {
    for result in results {
        assert!(!result.stdout.contains(secret), "secret sur stdout");
        assert!(!result.stderr.contains(secret), "secret sur stderr");
    }
}

impl SbWorld {
    /// Dit si ce secret apparaît dans un fichier du profil (base SQLite, journal WAL, verrou).
    pub fn profile_contains(&self, secret: &str) -> bool {
        directory_contains(self.home.path(), secret.as_bytes())
    }
}
