#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Fichier d'export : jamais d'écriture à travers un lien symbolique préparé à l'avance (audit 10).

use std::fs;
use std::os::unix::fs::{PermissionsExt, symlink};

use mira_core::maintenance::write_export_file;

/// Un lien symbolique placé au nom du temporaire historique n'est pas suivi
#[test]
fn regression_planted_symlink_is_never_written_through() {
    let dir = tempfile::tempdir().unwrap();
    let victim = dir.path().join("victim.txt");
    fs::write(&victim, "précieux").unwrap();
    let destination = dir.path().join("export.json");
    symlink(&victim, dir.path().join(".export.json.mira-part")).unwrap();
    write_export_file(&destination, b"{}").unwrap();
    assert_eq!(fs::read_to_string(&victim).unwrap(), "précieux");
    assert_eq!(fs::read(&destination).unwrap(), b"{}");
}

/// Le fichier final est privé, aucun temporaire ne reste, et deux exports se remplacent
#[test]
fn export_is_private_and_leaves_no_temporary_file() {
    let dir = tempfile::tempdir().unwrap();
    let destination = dir.path().join("export.json");
    write_export_file(&destination, b"un").unwrap();
    write_export_file(&destination, b"deux").unwrap();
    assert_eq!(fs::read(&destination).unwrap(), b"deux");
    let mode = fs::metadata(&destination).unwrap().permissions().mode();
    assert_eq!(mode & 0o777, 0o600);
    let names: Vec<_> = fs::read_dir(dir.path()).unwrap().collect();
    assert_eq!(names.len(), 1);
}

/// Un lien symbolique au nom de la destination est remplacé, jamais suivi
#[test]
fn regression_symlink_at_destination_is_replaced_not_followed() {
    let dir = tempfile::tempdir().unwrap();
    let victim = dir.path().join("victim.txt");
    fs::write(&victim, "précieux").unwrap();
    let destination = dir.path().join("export.json");
    symlink(&victim, &destination).unwrap();
    write_export_file(&destination, b"{}").unwrap();
    assert_eq!(fs::read_to_string(&victim).unwrap(), "précieux");
    assert_eq!(fs::read(&destination).unwrap(), b"{}");
}
