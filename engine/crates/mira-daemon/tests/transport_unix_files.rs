#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Écouteur Unix : socket périmée remplacée, socket vivante respectée, dossier ouvert refusé.

use std::os::unix::fs::{DirBuilderExt, PermissionsExt};
use std::os::unix::net::UnixListener;

use mira_daemon::transport::unix::{UnixSocketListener, check_directory_owner};

/// Un fichier de socket laissé par un processus disparu est remplacé.
#[tokio::test]
async fn a_stale_socket_file_is_replaced() {
    let dir = tempfile::tempdir().unwrap();
    let run = dir.path().join("run");
    std::fs::DirBuilder::new().mode(0o700).create(&run).unwrap();
    let path = run.join("mira.sock");
    drop(UnixListener::bind(&path).unwrap());
    assert!(path.exists());
    let listener = UnixSocketListener::bind(&path);
    assert!(listener.is_ok(), "{:?}", listener.err());
}

/// Une socket sur laquelle quelqu'un écoute n'est jamais volée.
#[tokio::test]
async fn a_live_socket_is_refused_with_addr_in_use() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("run/mira.sock");
    let _first = UnixSocketListener::bind(&path).unwrap();
    let second = UnixSocketListener::bind(&path).unwrap_err();
    assert_eq!(second.kind(), std::io::ErrorKind::AddrInUse);
    assert!(path.exists());
}

/// Un dossier ouvert au groupe ou aux autres est refusé, et un fichier ordinaire n'est jamais effacé.
#[tokio::test]
async fn a_shared_directory_or_a_regular_file_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let shared = dir.path().join("shared");
    std::fs::create_dir(&shared).unwrap();
    std::fs::set_permissions(&shared, std::fs::Permissions::from_mode(0o777)).unwrap();
    let error = UnixSocketListener::bind(&shared.join("mira.sock")).unwrap_err();
    assert_eq!(error.kind(), std::io::ErrorKind::PermissionDenied);
    let regular = dir.path().join("run/regular");
    std::fs::DirBuilder::new()
        .mode(0o700)
        .create(dir.path().join("run"))
        .unwrap();
    std::fs::write(&regular, "précieux").unwrap();
    let refused = UnixSocketListener::bind(&regular).unwrap_err();
    assert_eq!(refused.kind(), std::io::ErrorKind::AlreadyExists);
    assert_eq!(std::fs::read_to_string(&regular).unwrap(), "précieux");
}

/// Un verrou tenu par un autre démon qui démarre refuse la liaison, même sans fichier de socket (course au démarrage).
#[tokio::test]
async fn a_lock_held_elsewhere_refuses_the_bind_even_without_a_socket_file() {
    let dir = tempfile::tempdir().unwrap();
    let run = dir.path().join("run");
    std::fs::DirBuilder::new().mode(0o700).create(&run).unwrap();
    let lock = std::fs::File::create(run.join("mira.sock.lock")).unwrap();
    lock.try_lock().unwrap();
    let refused = UnixSocketListener::bind(&run.join("mira.sock")).unwrap_err();
    assert_eq!(refused.kind(), std::io::ErrorKind::AddrInUse);
    assert!(!run.join("mira.sock").exists());
}

/// Une socket que l'on ne peut pas sonder (droits retirés) n'est jamais supprimée : seul « connexion refusée » prouve qu'elle est périmée.
#[tokio::test]
async fn an_unprobeable_socket_file_is_not_removed() {
    let dir = tempfile::tempdir().unwrap();
    let run = dir.path().join("run");
    std::fs::DirBuilder::new().mode(0o700).create(&run).unwrap();
    let path = run.join("mira.sock");
    drop(UnixListener::bind(&path).unwrap());
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o000)).unwrap();
    let refused = UnixSocketListener::bind(&path).unwrap_err();
    assert_eq!(refused.kind(), std::io::ErrorKind::PermissionDenied);
    assert!(path.exists());
}

/// Un dossier qui n'appartient pas au démon est refusé, même en 0700.
#[test]
fn a_directory_owned_by_someone_else_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let own = std::os::unix::fs::MetadataExt::uid(&std::fs::metadata(dir.path()).unwrap());
    assert!(check_directory_owner(dir.path(), own).is_ok());
    let refused = check_directory_owner(dir.path(), own + 1).unwrap_err();
    assert_eq!(refused.kind(), std::io::ErrorKind::PermissionDenied);
}
