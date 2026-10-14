//! Recherche d'un secret dans tout ce qu'un scénario a produit : sorties du CLI et fichiers du profil.

use std::path::Path;

use super::sa_world::SaWorld;

/// Dit si un fichier quelconque sous ce dossier contient ces octets.
pub fn tree_contains(dir: &Path, needle: &str) -> bool {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return false;
    };
    entries.flatten().any(|entry| {
        let path = entry.path();
        if path.is_dir() {
            return tree_contains(&path, needle);
        }
        let bytes = std::fs::read(&path).unwrap_or_default();
        bytes
            .windows(needle.len())
            .any(|window| window == needle.as_bytes())
    })
}

impl SaWorld {
    /// Exige qu'aucun de ces secrets n'apparaisse dans une sortie du CLI ni dans un fichier du profil.
    pub fn assert_no_secret_anywhere(&self, secrets: &[&str]) {
        let log = self.transcript.lock().unwrap();
        for secret in secrets {
            assert!(!secret.is_empty());
            assert!(
                !log.iter().any(|text| text.contains(secret)),
                "secret dans une sortie du CLI"
            );
            assert!(
                !tree_contains(self.bench.home.path(), secret),
                "secret dans un fichier du profil"
            );
        }
    }
}
