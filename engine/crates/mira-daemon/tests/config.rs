#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Configuration du démon : chemins par défaut sous le profil, socket dans XDG_RUNTIME_DIR, fichier de clé.

use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use clap::Parser;
use mira_assembly::mira_vocabulary::ProfileId;
use mira_assembly::profile::{ProfileEnvironment, ProfilePaths};
use mira_daemon::config::{DaemonConfig, read_secrets_key, resolve_paths};

/// Environnement dont le dossier de données est `home`.
fn environment(home: &Path) -> ProfileEnvironment {
    ProfileEnvironment {
        mira_home: Some(home.to_path_buf()),
        xdg_data_home: None,
        home: None,
    }
}

/// Sans option, la socket et le jeton vivent sous le dossier du profil.
#[test]
fn defaults_resolve_socket_and_token_under_the_profile() {
    let home = tempfile::tempdir().unwrap();
    let config = DaemonConfig::try_parse_from(["mira-daemon", "--profile", "p1"]).unwrap();
    let env = environment(home.path());
    let paths = resolve_paths(&config, &env, None).unwrap();
    let root = ProfilePaths::resolve(&ProfileId::from_raw("p1"), &env)
        .unwrap()
        .root;
    assert_eq!(paths.socket, root.join("daemon.sock"));
    assert_eq!(paths.token, root.join("daemon.token"));
}

/// Avec XDG_RUNTIME_DIR, la socket va dans `mira-daemon/<profil>.sock` ; une option explicite l'emporte.
#[test]
fn socket_defaults_to_xdg_runtime_dir_when_set() {
    let home = tempfile::tempdir().unwrap();
    let env = environment(home.path());
    let config = DaemonConfig::try_parse_from(["mira-daemon"]).unwrap();
    let runtime = Path::new("/run/user/1000");
    let paths = resolve_paths(&config, &env, Some(runtime)).unwrap();
    assert_eq!(paths.socket, runtime.join("mira-daemon/default.sock"));
    let explicit = DaemonConfig::try_parse_from(["mira-daemon", "--socket", "/x/y.sock"]).unwrap();
    assert_eq!(
        resolve_paths(&explicit, &env, Some(runtime))
            .unwrap()
            .socket,
        Path::new("/x/y.sock")
    );
}

/// Un fichier de clé qui ne fait pas 32 octets est refusé.
#[test]
fn a_secrets_key_file_of_the_wrong_length_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("key");
    std::fs::write(&path, [1u8; 16]).unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).unwrap();
    assert!(read_secrets_key(&path).is_err());
    std::fs::write(&path, [1u8; 32]).unwrap();
    assert_eq!(read_secrets_key(&path).unwrap(), vec![1u8; 32]);
}

/// Un fichier de clé lisible par d'autres est refusé.
#[test]
fn a_secrets_key_file_readable_by_others_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("key");
    std::fs::write(&path, [1u8; 32]).unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap();
    assert!(read_secrets_key(&path).is_err());
}
