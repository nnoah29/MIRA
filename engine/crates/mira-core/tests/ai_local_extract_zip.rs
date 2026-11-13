#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Extraction durcie d'une archive `.zip` du serveur : mêmes garanties que pour le `.tar.gz`.

use mira_core::ai::local::extract_zip;

mod local_support;
use local_support::zip_archives::{ZipEntry, write_zip, zip_bytes};

/// Une archive normale s'extrait, bibliothèques comprises
#[test]
fn extracts_a_normal_zip() {
    let dir = tempfile::tempdir().unwrap();
    let archive = write_zip(
        dir.path(),
        &[
            ZipEntry::file("build/bin/model-server.exe", b"EXE"),
            ZipEntry::file("build/bin/lib.dll", b"LIB"),
        ],
    );
    let dest = dir.path().join("out");
    extract_zip(&archive, &dest, 1_000_000).unwrap();
    assert_eq!(
        std::fs::read(dest.join("build/bin/model-server.exe")).unwrap(),
        b"EXE"
    );
    assert_eq!(
        std::fs::read(dest.join("build/bin/lib.dll")).unwrap(),
        b"LIB"
    );
}

/// Chaque nom piégé est refusé, rien n'est écrit et le dossier final n'existe pas
#[test]
fn refuses_hostile_names() {
    for name in [
        "../evil.txt",
        "a/../../evil.txt",
        "/abs.txt",
        "C:/evil.txt",
        "C:evil.txt",
        "a\\..\\evil.txt",
        "\\\\host\\share\\evil.txt",
        "CON",
        "dir/NUL.txt",
        "a/b:stream",
    ] {
        let dir = tempfile::tempdir().unwrap();
        let archive = write_zip(dir.path(), &[ZipEntry::file(name, b"X")]);
        let dest = dir.path().join("out");
        let error = extract_zip(&archive, &dest, 1_000_000).unwrap_err();
        assert_eq!(error.code, "local_storage.corrupted", "{name}");
        assert!(
            !dest.exists() && !dir.path().join("evil.txt").exists(),
            "{name}"
        );
    }
}

/// Un lien symbolique est refusé, même vers un fichier voisin
#[test]
fn refuses_symlinks() {
    let dir = tempfile::tempdir().unwrap();
    let archive = write_zip(dir.path(), &[ZipEntry::symlink("a/link", "b")]);
    let dest = dir.path().join("out");
    let error = extract_zip(&archive, &dest, 1_000_000).unwrap_err();
    assert_eq!(error.code, "local_storage.corrupted");
    assert!(!dest.exists());
}

/// Le plafond d'octets décompressés annule tout, sans dossier à moitié rempli
#[test]
fn refuses_unpacked_size_over_the_cap() {
    let dir = tempfile::tempdir().unwrap();
    let archive = write_zip(
        dir.path(),
        &[
            ZipEntry::file("a.bin", &[7; 600]),
            ZipEntry::file("b.bin", &[7; 600]),
        ],
    );
    let dest = dir.path().join("out");
    let error = extract_zip(&archive, &dest, 1_000).unwrap_err();
    assert_eq!(error.code, "local_storage.corrupted");
    assert!(!dest.exists() && !dir.path().join("out.staging").exists());
}

/// Une archive tronquée est refusée proprement
#[test]
fn refuses_a_truncated_zip() {
    let dir = tempfile::tempdir().unwrap();
    let bytes = zip_bytes(&[ZipEntry::file("a.bin", &[7; 4000])]);
    let archive = dir.path().join("cut.zip");
    std::fs::write(&archive, &bytes[..bytes.len() / 2]).unwrap();
    let dest = dir.path().join("out");
    let error = extract_zip(&archive, &dest, 1_000_000).unwrap_err();
    assert_eq!(error.code, "local_storage.corrupted");
    assert!(!dest.exists());
}
