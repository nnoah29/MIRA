//! Démon lancé comme vrai processus : sorties lues par un fil, signaux envoyés par la commande `kill`.

use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::time::{Duration, Instant};

/// Processus `mira-daemon` en cours, avec les lignes qu'il écrit sur stdout et stderr.
pub struct RunningDaemon {
    child: Child,
    lines: Receiver<String>,
    pub output: Vec<String>,
}

impl RunningDaemon {
    /// Lance le binaire avec ces arguments ; stdout et stderr sont relus ligne à ligne.
    pub fn spawn(args: &[&str]) -> Self {
        let mut command = Command::new(env!("CARGO_BIN_EXE_mira-daemon"));
        command
            .args(args)
            .env_remove("XDG_RUNTIME_DIR")
            .env_remove("DBUS_SESSION_BUS_ADDRESS")
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

    /// Attend la ligne `mira-daemon prêt` (jusqu'à `wait`) ; panique si le processus finit avant.
    pub fn wait_ready(&mut self, wait: Duration) {
        let deadline = Instant::now() + wait;
        loop {
            let left = deadline.saturating_duration_since(Instant::now());
            match self.lines.recv_timeout(left) {
                Ok(line) => {
                    let ready = line == "mira-daemon prêt";
                    self.output.push(line);
                    if ready {
                        return;
                    }
                }
                Err(RecvTimeoutError::Timeout) => panic!("démon jamais prêt : {:?}", self.output),
                Err(RecvTimeoutError::Disconnected) => {
                    panic!("démon fini avant d'être prêt : {:?}", self.output)
                }
            }
        }
    }

    /// Envoie SIGTERM par la commande `kill`.
    pub fn terminate(&self) {
        let status = Command::new("kill")
            .args(["-TERM", &self.child.id().to_string()])
            .status()
            .unwrap();
        assert!(status.success());
    }

    /// Tue le processus sans préavis (SIGKILL) et attend sa fin.
    pub fn kill9(&mut self) {
        self.child.kill().unwrap();
        self.child.wait().unwrap();
    }

    /// Attend la fin du processus, puis rend son statut et tout ce qu'il a écrit.
    pub fn wait_exit(&mut self, wait: Duration) -> ExitStatus {
        let deadline = Instant::now() + wait;
        loop {
            if let Some(status) = self.child.try_wait().unwrap() {
                while let Ok(line) = self.lines.recv_timeout(Duration::from_millis(100)) {
                    self.output.push(line);
                }
                return status;
            }
            assert!(
                Instant::now() < deadline,
                "le démon ne s'arrête pas : {:?}",
                self.output
            );
            std::thread::sleep(Duration::from_millis(20));
        }
    }
}

impl Drop for RunningDaemon {
    /// Ne laisse jamais un démon orphelin après un test
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
