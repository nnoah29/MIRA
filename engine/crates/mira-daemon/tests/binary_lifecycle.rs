#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Le vrai binaire : démarrage, verrou de profil, signaux, socket périmée, TCP loopback, aucun jeton dans les sorties.

mod support;

use std::time::Duration;

use mira_protocol::{Frame, HelloRefusal};
use serde_json::json;
use support::process::RunningDaemon;
use support::process_args::{attach, base_args, start};
use support::{TestClient, valid_hello};
use tokio::net::TcpStream;

/// Le démon démarre et répond à `get_contract_info` sur sa socket.
#[tokio::test(flavor = "multi_thread")]
async fn the_daemon_starts_and_answers_contract_info_over_its_socket() {
    let dir = tempfile::tempdir().unwrap();
    let _daemon = start(dir.path(), &[]);
    let mut client = attach(dir.path()).await;
    let info = client
        .call(json!({"domain": "engine", "request": {"op": "get_contract_info"}}))
        .await;
    assert_eq!(info["ok"], true, "{info}");
}

/// Un second démon sur le même profil (même socket, ou une autre) sort avec le code 3 ; le premier répond toujours.
#[tokio::test(flavor = "multi_thread")]
async fn a_second_daemon_on_the_same_profile_exits_with_code_3_and_the_first_keeps_answering() {
    let dir = tempfile::tempdir().unwrap();
    let _first = start(dir.path(), &[]);
    let args = base_args(dir.path());
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    let mut second = RunningDaemon::spawn(&refs);
    let status = second.wait_exit(Duration::from_secs(60));
    assert_eq!(status.code(), Some(3), "{:?}", second.output);
    let mut client = attach(dir.path()).await;
    let info = client
        .call(json!({"domain": "engine", "request": {"op": "get_contract_info"}}))
        .await;
    assert_eq!(info["ok"], true);
}

/// SIGTERM arrête le démon avec le code 0 et supprime la socket.
#[tokio::test(flavor = "multi_thread")]
async fn sigterm_stops_the_daemon_with_code_0_and_removes_the_socket() {
    let dir = tempfile::tempdir().unwrap();
    let mut daemon = start(dir.path(), &[]);
    daemon.terminate();
    let status = daemon.wait_exit(Duration::from_secs(30));
    assert_eq!(status.code(), Some(0), "{:?}", daemon.output);
    assert!(!dir.path().join("run/mira.sock").exists());
}

/// Une socket laissée par un démon tué est remplacée au redémarrage.
#[tokio::test(flavor = "multi_thread")]
async fn a_socket_left_by_a_killed_daemon_is_replaced_at_restart() {
    let dir = tempfile::tempdir().unwrap();
    let mut first = start(dir.path(), &[]);
    first.kill9();
    assert!(
        dir.path().join("run/mira.sock").exists(),
        "le démon tué laisse sa socket"
    );
    let _second = start(dir.path(), &[]);
    let mut client = attach(dir.path()).await;
    let info = client
        .call(json!({"domain": "engine", "request": {"op": "get_contract_info"}}))
        .await;
    assert_eq!(info["ok"], true);
}

/// `--tcp-port 0` écoute sur le loopback, annonce le port et exige le jeton.
#[tokio::test(flavor = "multi_thread")]
async fn tcp_port_0_listens_on_loopback_and_requires_the_token() {
    let dir = tempfile::tempdir().unwrap();
    let daemon = start(dir.path(), &["--tcp-port", "0"]);
    let line = daemon
        .output
        .iter()
        .find(|l| l.starts_with("tcp_port="))
        .expect("port annoncé");
    let port: u16 = line["tcp_port=".len()..].parse().unwrap();
    let wrong = TcpStream::connect(("127.0.0.1", port)).await.unwrap();
    let mut client = TestClient::over(Box::new(wrong));
    client.send(valid_hello(&"0".repeat(64))).await;
    let reply = client.recv().await.unwrap();
    assert!(matches!(
        reply,
        Frame::HelloErr {
            reason: HelloRefusal::Auth
        }
    ));
    let token = std::fs::read_to_string(dir.path().join("daemon.token")).unwrap();
    let right = TcpStream::connect(("127.0.0.1", port)).await.unwrap();
    TestClient::over(Box::new(right))
        .hello_ok(token.trim())
        .await;
}

/// Ni stdout ni stderr ne contiennent le jeton.
#[tokio::test(flavor = "multi_thread")]
async fn stdout_and_stderr_never_contain_the_token() {
    let dir = tempfile::tempdir().unwrap();
    let mut daemon = start(dir.path(), &["--tcp-port", "0"]);
    let token = std::fs::read_to_string(dir.path().join("daemon.token")).unwrap();
    daemon.terminate();
    daemon.wait_exit(Duration::from_secs(30));
    assert!(!daemon.output.iter().any(|line| line.contains(token.trim())));
}
