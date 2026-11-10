#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Extraction durcie de l'archive du serveur : arborescence normale, entrées piégées refusées.

use mira_core::ai::local::extract_tar_gz;

mod local_support;
use local_support::archives::{Entry, write_archive};

/// Une archive normale s'extrait, bibliothèques comprises
#[test]
fn extracts_a_normal_archive() {
    let dir = tempfile::tempdir().unwrap();
    let archive = write_archive(
        dir.path(),
        &[
            Entry::file("build/bin/llama-server", b"EXE"),
            Entry::file("build/bin/libx.so", b"LIB"),
        ],
    );
    let dest = dir.path().join("out");
    extract_tar_gz(&archive, &dest, 1_000_000).unwrap();
    assert_eq!(
        std::fs::read(dest.join("build/bin/llama-server")).unwrap(),
        b"EXE"
    );
    assert_eq!(
        std::fs::read(dest.join("build/bin/libx.so")).unwrap(),
        b"LIB"
    );
}

/// Une entrée `..` est refusée et rien n'est écrit hors du dossier
#[test]
fn refuses_dotdot_entries() {
    let dir = tempfile::tempdir().unwrap();
    let archive = write_archive(dir.path(), &[Entry::file("../evil.txt", b"X")]);
    let dest = dir.path().join("out");
    let error = extract_tar_gz(&archive, &dest, 1_000_000).unwrap_err();
    assert_eq!(error.code, "local_storage.corrupted");
    assert!(!dir.path().join("evil.txt").exists() && !dest.exists());
}

/// Un chemin absolu est refusé
#[test]
fn refuses_absolute_paths() {
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path().join("abs.txt");
    let archive = write_archive(dir.path(), &[Entry::file(target.to_str().unwrap(), b"X")]);
    let dest = dir.path().join("out");
    let error = extract_tar_gz(&archive, &dest, 1_000_000).unwrap_err();
    assert_eq!(error.code, "local_storage.corrupted");
    assert!(!target.exists() && !dest.exists());
}

/// Un lien symbolique qui sort du dossier est refusé
#[test]
fn refuses_symlink_escaping_the_folder() {
    let dir = tempfile::tempdir().unwrap();
    let archive = write_archive(dir.path(), &[Entry::symlink("a/escape", "../../outside")]);
    let dest = dir.path().join("out");
    let error = extract_tar_gz(&archive, &dest, 1_000_000).unwrap_err();
    assert_eq!(error.code, "local_storage.corrupted");
    assert!(!dest.exists());
}

/// Une archive dont la taille décompressée dépasse la limite est refusée
#[test]
fn refuses_decompression_bomb_over_the_limit() {
    let dir = tempfile::tempdir().unwrap();
    let archive = write_archive(dir.path(), &[Entry::file("big.bin", &vec![0u8; 5_000])]);
    let dest = dir.path().join("out");
    let error = extract_tar_gz(&archive, &dest, 1_000).unwrap_err();
    assert_eq!(error.code, "local_storage.corrupted");
    assert!(!dest.exists());
}
