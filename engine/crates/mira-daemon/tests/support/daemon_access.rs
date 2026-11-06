//! Accès au démon jetable : socket, jeton, contexte, dossier et tâche de service.

use std::path::Path;
use std::sync::Arc;

use mira_daemon::server::ServerContext;
use tokio::task::JoinHandle;

use super::TestDaemon;

impl TestDaemon {
    /// Chemin de la socket Unix.
    pub fn socket_path(&self) -> &Path {
        &self.socket
    }

    /// Jeton local, en hexadécimal.
    pub fn token(&self) -> &str {
        &self.token
    }

    /// Contexte partagé du serveur.
    pub fn context(&self) -> Arc<ServerContext> {
        self.context.clone()
    }

    /// Dossier temporaire qui contient la socket et le jeton.
    pub fn dir(&self) -> &Path {
        self.dir.path()
    }

    /// Tâche `serve`, pour attendre sa fin après un arrêt.
    pub fn task(&mut self) -> &mut JoinHandle<()> {
        &mut self.task
    }
}
