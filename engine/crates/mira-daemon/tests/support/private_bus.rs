//! Bus de session privé pour les tests D-Bus : un `dbus-daemon` lancé exprès, jamais celui de l'utilisateur.

use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};

/// Bus de session jetable ; le processus est tué à la fin du test.
pub struct PrivateBus {
    child: Child,
    pub address: String,
}

impl PrivateBus {
    /// Lance `dbus-daemon` ; échoue clairement s'il manque, un test D-Bus ne s'ignore jamais.
    pub fn start() -> Self {
        let mut child = Command::new("dbus-daemon")
            .args(["--session", "--nofork", "--print-address=1"])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("dbus-daemon est requis pour les tests D-Bus");
        let stdout = child.stdout.take().unwrap();
        let mut address = String::new();
        BufReader::new(stdout).read_line(&mut address).unwrap();
        Self {
            child,
            address: address.trim().to_string(),
        }
    }
}

impl Drop for PrivateBus {
    /// Arrête le bus privé.
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
