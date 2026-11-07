#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Écouteur Unix : permissions de la socket et de son dossier, uid du pair, nettoyage à la fermeture.

use std::os::unix::fs::PermissionsExt;

use mira_daemon::transport::unix::{UnixSocketListener, peer_is_owner};
use mira_daemon::transport::{Listener, Trust};
use tokio::net::UnixStream;

/// Le fichier de socket est en 0600 et le dossier créé pour lui en 0700.
#[tokio::test]
async fn the_socket_file_is_0600_and_its_directory_0700() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("run/mira.sock");
    let _listener = UnixSocketListener::bind(&path).unwrap();
    let file = std::fs::metadata(&path).unwrap().permissions().mode();
    let folder = std::fs::metadata(path.parent().unwrap())
        .unwrap()
        .permissions()
        .mode();
    assert_eq!(file & 0o777, 0o600);
    assert_eq!(folder & 0o777, 0o700);
}

/// Un client du même utilisateur est accepté, avec son uid.
#[tokio::test]
async fn a_client_of_the_same_user_is_accepted_with_its_uid() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("run/mira.sock");
    let listener = UnixSocketListener::bind(&path).unwrap();
    let _client = UnixStream::connect(&path).await.unwrap();
    let accepted = listener.accept().await.unwrap();
    let own = std::fs::metadata(&path).unwrap();
    let uid = std::os::unix::fs::MetadataExt::uid(&own);
    assert!(matches!(accepted.trust, Trust::LocalUser(Some(found)) if found == uid));
}

/// Fermer l'écouteur supprime le fichier de socket.
#[tokio::test]
async fn dropping_the_listener_removes_the_socket_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("run/mira.sock");
    let listener = UnixSocketListener::bind(&path).unwrap();
    assert!(path.exists());
    drop(listener);
    assert!(!path.exists());
}

/// Seul l'uid propriétaire passe le contrôle du pair.
#[test]
fn only_the_owner_uid_passes_peer_check() {
    assert!(peer_is_owner(1000, 1000));
    assert!(!peer_is_owner(1001, 1000));
    assert!(!peer_is_owner(0, 1000));
}
