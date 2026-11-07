//! Lancement du démon avec des variables d'environnement choisies (bus de session privé, langue, lanceur de l'app).

use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::sync::mpsc;

use super::process::RunningDaemon;

impl RunningDaemon {
    /// Comme `spawn`, avec ces variables en plus ; `XDG_RUNTIME_DIR` et le bus de session de l'utilisateur sont retirés.
    pub fn spawn_with_env(args: &[&str], env: &[(&str, &str)]) -> Self {
        let mut command = Command::new(env!("CARGO_BIN_EXE_mira-daemon"));
        command
            .args(args)
            .env_remove("XDG_RUNTIME_DIR")
            .env_remove("DBUS_SESSION_BUS_ADDRESS")
            .envs(env.iter().copied())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let mut child = command.spawn().unwrap();
        let (sender, lines) = mpsc::channel();
        let stdout = child.stdout.take().unwrap();
        let stderr = child.stderr.take().unwrap();
        for pipe in [
            Box::new(stdout) as Box<dyn std::io::Read + Send>,
            Box::new(stderr),
        ] {
            let sender = sender.clone();
            std::thread::spawn(move || {
                for line in BufReader::new(pipe).lines().map_while(Result::ok) {
                    let _ = sender.send(line);
                }
            });
        }
        Self {
            child,
            lines,
            output: Vec::new(),
        }
    }
}
