#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Démon lancé à la demande : il s'arrête quand le dernier client est parti depuis le délai de grâce, jamais sans l'option.

mod support;

use std::time::Duration;

use serde_json::json;
use support::process_args::{attach, start};

const WAIT: Duration = Duration::from_secs(30);

/// Requête bénigne pour vérifier que le démon répond encore.
fn info() -> serde_json::Value {
    json!({"domain": "engine", "request": {"op": "get_contract_info"}})
}

/// Le dernier client parti, le démon s'arrête après la grâce avec le code 0 et retire sa socket.
#[tokio::test(flavor = "multi_thread")]
async fn exits_after_the_last_client_leaves_and_the_grace_passes() {
    let dir = tempfile::tempdir().unwrap();
    let mut daemon = start(dir.path(), &["--exit-when-idle", "--idle-grace-secs", "3"]);
    let client = attach(dir.path()).await;
    drop(client);
    let status = daemon.wait_exit(WAIT);
    assert_eq!(status.code(), Some(0), "{:?}", daemon.output);
    assert!(!dir.path().join("run/mira.sock").exists());
}

/// Un client qui revient avant la fin de la grâce garde le démon en vie, même au-delà de la grâce.
#[tokio::test(flavor = "multi_thread")]
async fn a_new_client_within_the_grace_keeps_the_daemon_alive() {
    let dir = tempfile::tempdir().unwrap();
    let mut daemon = start(dir.path(), &["--exit-when-idle", "--idle-grace-secs", "3"]);
    drop(attach(dir.path()).await);
    tokio::time::sleep(Duration::from_millis(1200)).await;
    let mut second = attach(dir.path()).await;
    tokio::time::sleep(Duration::from_secs(4)).await;
    assert_eq!(second.call(info()).await["ok"], true);
    drop(second);
    let status = daemon.wait_exit(WAIT);
    assert_eq!(status.code(), Some(0), "{:?}", daemon.output);
}

/// Sans `--exit-when-idle`, le démon reste en vie même sans client au-delà de la grâce.
#[tokio::test(flavor = "multi_thread")]
async fn without_the_option_the_daemon_never_exits_on_idle() {
    let dir = tempfile::tempdir().unwrap();
    let _daemon = start(dir.path(), &["--idle-grace-secs", "1"]);
    drop(attach(dir.path()).await);
    tokio::time::sleep(Duration::from_secs(3)).await;
    assert_eq!(attach(dir.path()).await.call(info()).await["ok"], true);
}
