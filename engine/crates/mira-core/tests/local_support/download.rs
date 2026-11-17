#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, dead_code)]
//! Outils des tests de téléchargement : contenu, somme, tâche prête et dossier temporaire.

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use mira_core::ai::local::DownloadJob;
use mira_core::ports::SystemResources;
use sha2::{Digest, Sha256};

/// Contenu de test déterministe de la taille voulue
pub fn content(len: usize) -> Vec<u8> {
    (0..len)
        .map(|i| u8::try_from(i * 7 % 251).unwrap())
        .collect()
}

/// Somme SHA-256 en hexadécimal minuscule
pub fn sha_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// Drapeau d'annulation non levé
pub fn no_cancel() -> Arc<AtomicBool> {
    Arc::new(AtomicBool::new(false))
}

/// Chemin du fichier final dans un dossier temporaire
pub fn dest_in(dir: &tempfile::TempDir) -> PathBuf {
    dir.path().join("models").join("m.bin")
}

/// Tâche prête : adresse, somme et taille épinglées, destination, ressources abondantes
pub fn job<'a>(
    url: &'a str,
    bytes: &'a [u8],
    sha: &'a str,
    dest: &'a std::path::Path,
    res: &'a dyn SystemResources,
) -> DownloadJob<'a> {
    DownloadJob {
        url,
        sha256: sha,
        size: bytes.len() as u64,
        dest,
        resources: res,
    }
}
