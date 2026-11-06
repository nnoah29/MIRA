#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Écouteur TCP : loopback seulement, port libre au besoin, pair local sans uid.

use mira_daemon::transport::tcp::LoopbackTcpListener;
use mira_daemon::transport::{Listener, Trust};
use tokio::net::TcpStream;

/// L'écouteur n'est lié qu'à 127.0.0.1, port 0 donne un port libre.
#[tokio::test]
async fn the_listener_is_bound_on_loopback_and_port_0_picks_a_free_port() {
    let listener = LoopbackTcpListener::bind(0).unwrap();
    assert_ne!(listener.local_port(), 0);
    assert!(listener.describe().starts_with("tcp:127.0.0.1:"));
}

/// Un client se connecte et est traité comme utilisateur local sans uid connu.
#[tokio::test]
async fn a_client_connects_and_is_local_user_without_uid() {
    let listener = LoopbackTcpListener::bind(0).unwrap();
    let _client = TcpStream::connect(("127.0.0.1", listener.local_port()))
        .await
        .unwrap();
    let accepted = listener.accept().await.unwrap();
    assert!(matches!(accepted.trust, Trust::LocalUser(None)));
}

/// Aucune interface autre que le loopback n'accepte de connexion.
#[tokio::test]
async fn other_interfaces_are_not_listening() {
    let listener = LoopbackTcpListener::bind(0).unwrap();
    let address = listener.local_addr();
    assert!(address.ip().is_loopback());
    assert!(!address.ip().is_unspecified());
}
