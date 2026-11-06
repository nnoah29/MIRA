#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Le binaire sans options de chemin, et un second démon qui n'est arrêté que par le verrou du profil.

mod support;

use std::time::Duration;

use serde_json::json;
use support::TestClient;
use support::process::RunningDaemon;
use support::process_args::{base_args, default_args, start};

/// Sans `XDG_RUNTIME_DIR` ni `--socket`, un profil neuf démarre : la socket et le jeton vivent sous le profil.
#[tokio::test(flavor = "multi_thread")]
async fn a_new_profile_starts_with_default_paths_and_no_xdg_runtime_dir() {
    let dir = tempfile::tempdir().unwrap();
    let args = default_args(dir.path());
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    let mut daemon = RunningDaemon::spawn(&refs);
    daemon.wait_ready(Duration::from_secs(60));
    let root = dir.path().join("home/profiles/default");
    let token = std::fs::read_to_string(root.join("daemon.token")).unwrap();
    let mut client = TestClient::connect_unix(&root.join("daemon.sock")).await;
    client.hello_ok(token.trim()).await;
}

/// Un second démon sur une AUTRE socket est arrêté par le verrou du profil (code 3) ; le premier répond toujours.
#[tokio::test(flavor = "multi_thread")]
async fn a_second_daemon_on_another_socket_is_stopped_by_the_profile_lock() {
    let dir = tempfile::tempdir().unwrap();
    let _first = start(dir.path(), &[]);
    let mut args = base_args(dir.path());
    let socket = args.iter().position(|a| a == "--socket").unwrap() + 1;
    args[socket] = dir
        .path()
        .join("run/other.sock")
        .to_str()
        .unwrap()
        .to_string();
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    let mut second = RunningDaemon::spawn(&refs);
    let status = second.wait_exit(Duration::from_secs(60));
    assert_eq!(status.code(), Some(3), "{:?}", second.output);
    assert!(
        second.output.iter().any(|l| l.contains("déjà utilisé")),
        "{:?}",
        second.output
    );
    let mut client = support::process_args::attach(dir.path()).await;
    let info = client
        .call(json!({"domain": "engine", "request": {"op": "get_contract_info"}}))
        .await;
    assert_eq!(info["ok"], true);
}
