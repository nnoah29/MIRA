#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Écouteur Unix : socket périmée remplacée, socket vivante respectée, dossier ouvert refusé.

use std::os::unix::fs::{DirBuilderExt, PermissionsExt};
use std::os::unix::net::UnixListener;

use mira_daemon::transport::unix::UnixSocketListener;

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
