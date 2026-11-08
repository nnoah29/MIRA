#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Le vrai binaire publie son port TCP dans `daemon.port` et le retire à l'arrêt.

mod support;

use std::time::Duration;

use support::process_args::start;

/// Le port annoncé sur stdout est celui du fichier, qui disparaît avec le démon.
#[tokio::test(flavor = "multi_thread")]
async fn the_announced_tcp_port_is_published_then_removed_at_shutdown() {
    let dir = tempfile::tempdir().unwrap();
    let mut daemon = start(dir.path(), &["--tcp-port", "0"]);
    let line = daemon
        .output
        .iter()
        .find(|l| l.starts_with("tcp_port="))
        .expect("port annoncé");
    let announced = line["tcp_port=".len()..].to_string();
    let file = dir.path().join("home/profiles/default/daemon.port");
    assert_eq!(std::fs::read_to_string(&file).unwrap().trim(), announced);
    daemon.terminate();
    let status = daemon.wait_exit(Duration::from_secs(30));
    assert_eq!(status.code(), Some(0), "{:?}", daemon.output);
    assert!(!file.exists());
}
