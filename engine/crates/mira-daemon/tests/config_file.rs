#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Fichier de configuration du démon : démarrage seulement, la ligne de commande l'emporte, erreurs à message fixe.

use std::path::Path;

use clap::Parser;
use mira_assembly::profile::ProfileEnvironment;
use mira_daemon::config::{DaemonConfig, FileConfig, config_file_path, read_file_config};

/// Environnement dont le dossier de données est `home`.
fn environment(home: &Path) -> ProfileEnvironment {
    ProfileEnvironment {
        mira_home: Some(home.to_path_buf()),
        xdg_data_home: None,
        home: None,
    }
}

/// Écrit `daemon.toml` avec ce contenu dans un dossier jetable et rend son chemin.
fn write_file(dir: &Path, content: &[u8]) -> std::path::PathBuf {
    let path = dir.join("daemon.toml");
    std::fs::write(&path, content).unwrap();
    path
}

/// Sans fichier, la configuration est vide.
#[test]
fn absent_file_gives_defaults() {
    let dir = tempfile::tempdir().unwrap();
    let read = read_file_config(&dir.path().join("daemon.toml")).unwrap();
    assert_eq!(read, FileConfig::default());
}

/// Le fichier remplit ce que la ligne de commande n'a pas donné.
#[test]
fn file_values_fill_missing_options() {
    let dir = tempfile::tempdir().unwrap();
    let text = "profile = \"p2\"\nhome = \"/h\"\nsocket = \"/s.sock\"\ntoken_file = \"/t\"\n";
    let file = read_file_config(&write_file(dir.path(), text.as_bytes())).unwrap();
    let merged = DaemonConfig::try_parse_from(["mira-daemon"])
        .unwrap()
        .merged_with(file);
    assert_eq!(merged.profile(), "p2");
    assert_eq!(merged.home.as_deref(), Some(Path::new("/h")));
    assert_eq!(merged.socket.as_deref(), Some(Path::new("/s.sock")));
    assert_eq!(merged.token_file.as_deref(), Some(Path::new("/t")));
}

/// Une option de la ligne de commande l'emporte sur le fichier.
#[test]
fn command_line_wins_over_file() {
    let dir = tempfile::tempdir().unwrap();
    let text = "profile = \"p2\"\nhome = \"/h\"\nsocket = \"/s.sock\"\n";
    let file = read_file_config(&write_file(dir.path(), text.as_bytes())).unwrap();
    let args = ["mira-daemon", "--profile", "cli", "--socket", "/cli.sock"];
    let merged = DaemonConfig::try_parse_from(args)
        .unwrap()
        .merged_with(file);
    assert_eq!(merged.profile(), "cli");
    assert_eq!(merged.socket.as_deref(), Some(Path::new("/cli.sock")));
    assert_eq!(merged.home.as_deref(), Some(Path::new("/h")));
}

/// Le fichier est à la racine du dossier MIRA, à côté de `profiles/`.
#[test]
fn config_file_lives_next_to_the_profiles_folder() {
    let home = tempfile::tempdir().unwrap();
    let path = config_file_path(&environment(home.path())).unwrap();
    assert_eq!(path, home.path().join("daemon.toml"));
}
