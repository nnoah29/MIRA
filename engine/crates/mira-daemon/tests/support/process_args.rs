//! Lancement du démon de test : arguments communs, démarrage jusqu'au « prêt », client attaché par sa socket.

use std::path::Path;
use std::time::Duration;

use super::TestClient;
use super::process::RunningDaemon;

const READY: Duration = Duration::from_secs(60);

/// Arguments communs : profil jetable sous `dir`, trousseau chiffré, socket et jeton explicites.
pub fn base_args(dir: &Path) -> Vec<String> {
    let key = dir.join("secrets.key");
    if !key.exists() {
        std::fs::write(&key, [7u8; 32]).unwrap();
        std::fs::set_permissions(&key, std::os::unix::fs::PermissionsExt::from_mode(0o600))
            .unwrap();
    }
    let path = |name: &str| dir.join(name).to_str().unwrap().to_string();
    vec![
        "--home".into(),
        path("home"),
        "--secrets-key-file".into(),
        path("secrets.key"),
        "--socket".into(),
        path("run/mira.sock"),
        "--token-file".into(),
        path("daemon.token"),
    ]
}

/// Lance le démon sur `dir` avec ces arguments en plus, et attend qu'il soit prêt.
pub fn start(dir: &Path, extra: &[&str]) -> RunningDaemon {
    let mut args = base_args(dir);
    args.extend(extra.iter().map(|a| a.to_string()));
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    let mut daemon = RunningDaemon::spawn(&refs);
    daemon.wait_ready(READY);
    daemon
}

/// Se connecte à la socket de `dir` avec le jeton écrit par le démon et fait `hello`.
pub async fn attach(dir: &Path) -> TestClient {
    let token = std::fs::read_to_string(dir.join("daemon.token")).unwrap();
    let mut client = TestClient::connect_unix(&dir.join("run/mira.sock")).await;
    client.hello_ok(token.trim()).await;
    client
}

/// Arguments minimaux : seulement le profil jetable et le trousseau chiffré ; socket et jeton prennent leur défaut.
pub fn default_args(dir: &Path) -> Vec<String> {
    let mut args = base_args(dir);
    args.truncate(4);
    args
}
