#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Fichier de configuration du démon : un fichier invalide est refusé avec un message fixe.

use std::path::Path;

use mira_daemon::config::read_file_config;

/// Écrit `daemon.toml` avec ce contenu dans un dossier jetable et rend son chemin.
fn write_file(dir: &Path, content: &[u8]) -> std::path::PathBuf {
    let path = dir.join("daemon.toml");
    std::fs::write(&path, content).unwrap();
    path
}

/// Une clé inconnue est refusée sans rien révéler du fichier.
#[test]
fn unknown_key_is_refused_with_fixed_message() {
    let dir = tempfile::tempdir().unwrap();
    let path = write_file(dir.path(), b"mot_de_passe_secret = \"abcxyz\"\n");
    let error = read_file_config(&path).unwrap_err();
    let shown = format!("{error:?} {error}");
    assert!(!shown.contains("abcxyz") && !shown.contains("mot_de_passe_secret"));
    assert!(!shown.contains(dir.path().to_str().unwrap()));
}

/// Un fichier de plus de 64 Kio est refusé.
#[test]
fn oversized_file_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let path = write_file(dir.path(), "# x\n".repeat(20_000).as_bytes());
    assert!(read_file_config(&path).is_err());
}

/// Un contenu non UTF-8 ou un chemin illisible sont refusés sans fuite du chemin.
#[test]
fn unreadable_file_is_refused_without_leaking_path() {
    let dir = tempfile::tempdir().unwrap();
    let binary = write_file(dir.path(), &[0xff, 0xfe, 0x00]);
    assert!(read_file_config(&binary).is_err());
    let folder = dir.path().join("sous-dossier");
    std::fs::create_dir(&folder).unwrap();
    let error = read_file_config(&folder).unwrap_err();
    assert!(!format!("{error:?} {error}").contains(folder.to_str().unwrap()));
}
