//! Processus du serveur : lancement à environnement réduit, sondage de l'état prêt, arrêt par signal.

use std::path::Path;
use std::process::Stdio;
use std::time::{Duration, Instant};

use mira_vocabulary::error::codes::server;
use mira_vocabulary::{ErrorCategory, MiraError};
use tokio::process::{Child, Command};

use super::os_process::{ask_to_stop, tune_command};

/// Lance l'exécutable sans hériter de l'environnement (seul `PATH` passe), sorties jetées.
pub(super) fn spawn(executable: &Path, args: &[String]) -> Result<Child, MiraError> {
    let mut command = Command::new(executable);
    command
        .args(args)
        .env_clear()
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .kill_on_drop(true);
    if let Some(path) = std::env::var_os("PATH") {
        command.env("PATH", path);
    }
    if let Some(folder) = executable.parent() {
        command.current_dir(folder);
    }
    tune_command(&mut command);
    command
        .spawn()
        .map_err(|_| failure("le serveur local n'a pas pu être lancé"))
}

/// Erreur d'exécution du serveur, avec un message sans détail.
pub(super) fn failure(message: &str) -> MiraError {
    MiraError::new(
        ErrorCategory::Server,
        server::AI_LOCAL_RUNTIME_FAILED,
        message,
    )
}

/// Erreur : le serveur n'est pas prêt dans le délai.
pub(super) fn timeout_error() -> MiraError {
    MiraError::new(
        ErrorCategory::Server,
        server::AI_LOCAL_START_TIMEOUT,
        "le modèle local n'est pas prêt dans le délai imparti",
    )
}

/// Attend que `/health` réponde 200 ; sortie du processus ou délai dépassé sont des erreurs.
pub(super) async fn wait_ready(
    child: &mut Child,
    base_url: &str,
    health_path: &str,
    timeout: Duration,
    low_memory: bool,
) -> Result<(), MiraError> {
    let client = reqwest::Client::builder()
        .no_proxy()
        .build()
        .map_err(|_| timeout_error())?;
    let deadline = Instant::now() + timeout;
    loop {
        if matches!(child.try_wait(), Ok(Some(_))) {
            return Err(failure(exit_message(low_memory)));
        }
        let probe = client.get(format!("{base_url}{health_path}")).send().await;
        if probe.is_ok_and(|r| r.status().is_success()) {
            return Ok(());
        }
        if Instant::now() >= deadline {
            return Err(timeout_error());
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

/// Message de sortie du serveur ; il évoque la mémoire quand elle était justement juste.
fn exit_message(low_memory: bool) -> &'static str {
    if low_memory {
        "le serveur local s'est arrêté : la mémoire disponible était probablement insuffisante"
    } else {
        "le serveur local s'est arrêté"
    }
}

/// Arrête le processus : SIGTERM, puis SIGKILL après cinq secondes ; le processus est toujours récolté.
pub(super) async fn terminate(child: &mut Child) {
    let asked = ask_to_stop(child);
    if asked
        && tokio::time::timeout(Duration::from_secs(5), child.wait())
            .await
            .is_ok()
    {
        return;
    }
    let _ = child.kill().await;
}
