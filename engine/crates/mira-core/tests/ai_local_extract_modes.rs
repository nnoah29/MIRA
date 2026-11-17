#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Extraction durcie : liens relatifs internes admis, modes fixés par le moteur.

use std::os::unix::fs::PermissionsExt;

use mira_core::ai::local::extract_tar_gz;

mod local_support;
use local_support::archives::{Entry, write_archive};

/// Un lien relatif qui reste dans le dossier est accepté
#[test]
fn allows_a_relative_symlink_inside_the_folder() {
    let dir = tempfile::tempdir().unwrap();
    let archive = write_archive(
        dir.path(),
        &[
            Entry::file("lib/libx.so.1", b"LIB"),
            Entry::symlink("lib/libx.so", "libx.so.1"),
        ],
    );
    let dest = dir.path().join("out");
    extract_tar_gz(&archive, &dest, 1_000_000).unwrap();
    assert_eq!(std::fs::read(dest.join("lib/libx.so")).unwrap(), b"LIB");
}

/// Aucun fichier extrait n'est modifiable par tous
#[test]
fn extracted_files_are_not_world_writable() {
    let dir = tempfile::tempdir().unwrap();
    let archive = write_archive(dir.path(), &[Entry::file("f.bin", b"X")]);
    let dest = dir.path().join("out");
    extract_tar_gz(&archive, &dest, 1_000_000).unwrap();
    let mode = std::fs::metadata(dest.join("f.bin"))
        .unwrap()
        .permissions()
        .mode();
    assert_eq!(mode & 0o002, 0);
}
