#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, dead_code)]
//! Faux serveur Python et gabarit d'arguments des tests du gestionnaire.

use std::path::Path;

const FAKE: &str = include_str!("fake_server.py");

/// Écrit le faux serveur exécutable et les fichiers de modèle factices
pub fn install_fake(dir: &Path) {
    let exe = dir.join("fake-server");
    std::fs::write(&exe, FAKE).unwrap();
    std::fs::set_permissions(&exe, std::os::unix::fs::PermissionsExt::from_mode(0o755)).unwrap();
    std::fs::create_dir_all(dir.join("models")).unwrap();
    for name in ["mini.bin", "big.bin"] {
        std::fs::write(dir.join("models").join(name), b"MODEL").unwrap();
    }
}

/// Gabarit d'arguments du faux serveur, suivi des options propres au faux
pub fn fake_args(log: &Path, extra: &[&str]) -> Vec<String> {
    let mut args: Vec<String> = [
        "--model",
        "{model}",
        "--host",
        "{host}",
        "--port",
        "{port}",
        "--api-key-file",
        "{api_key_file}",
        "--ctx-size",
        "{ctx}",
        "--threads",
        "{threads}",
        "--alias",
        "{alias}",
        "--fake-log",
    ]
    .iter()
    .map(|a| (*a).to_string())
    .collect();
    args.push(log.to_string_lossy().into_owned());
    args.extend(extra.iter().map(|a| (*a).to_string()));
    args
}
