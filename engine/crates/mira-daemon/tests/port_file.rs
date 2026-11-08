//! Fichier du port : publié à côté du jeton, un seul démon à la fois, retiré à l'arrêt.

use mira_daemon::transport::port_file::PortFile;

/// Le port publié se relit tel quel et le fichier disparaît avec le démon.
#[test]
fn published_port_is_readable_then_removed() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("daemon.port");
    let file = PortFile::publish(&path, 43210).unwrap();
    assert_eq!(std::fs::read_to_string(&path).unwrap().trim(), "43210");
    drop(file);
    assert!(!path.exists());
}

/// Un port périmé laissé par un démon planté est remplacé.
#[test]
fn stale_port_file_is_replaced() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("daemon.port");
    std::fs::write(&path, "1").unwrap();
    let file = PortFile::publish(&path, 2222).unwrap();
    assert_eq!(std::fs::read_to_string(&path).unwrap().trim(), "2222");
    drop(file);
}

/// Un second démon ne peut pas publier tant que le premier vit, et reçoit AddrInUse.
#[cfg(windows)]
#[test]
fn second_publisher_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("daemon.port");
    let first = PortFile::publish(&path, 1000).unwrap();
    let second = PortFile::publish(&path, 2000).unwrap_err();
    assert_eq!(second.kind(), std::io::ErrorKind::AddrInUse);
    drop(first);
}

/// Le fichier du port se résout sous le dossier du profil, à côté du jeton.
#[test]
fn port_path_sits_beside_the_token() {
    let home = tempfile::tempdir().unwrap();
    let env = mira_assembly::profile::ProfileEnvironment {
        mira_home: Some(home.path().to_path_buf()),
        xdg_data_home: None,
        home: None,
    };
    let config: mira_daemon::config::DaemonConfig =
        clap::Parser::try_parse_from(["mira-daemon"]).unwrap();
    let paths = mira_daemon::config::resolve_paths(&config, &env, None).unwrap();
    assert_eq!(paths.port.file_name().unwrap(), "daemon.port");
    assert_eq!(paths.port.parent(), paths.token.parent());
}
