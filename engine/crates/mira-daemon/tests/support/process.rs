//! Démon lancé comme vrai processus : sorties lues par un fil, signaux envoyés par la commande `kill`.

use std::path::Path;
use std::process::{Child, Command, ExitStatus};
use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::time::{Duration, Instant};

/// Processus `mira-daemon` en cours, avec les lignes qu'il écrit sur stdout et stderr.
pub struct RunningDaemon {
    pub(super) child: Child,
    pub(super) lines: Receiver<String>,
    pub output: Vec<String>,
}

impl RunningDaemon {
    /// Lance le binaire avec ces arguments ; stdout et stderr sont relus ligne à ligne.
    pub fn spawn(args: &[&str]) -> Self {
        Self::spawn_with_env(args, &[])
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
