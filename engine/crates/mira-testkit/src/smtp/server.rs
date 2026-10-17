//! Handle du faux serveur SMTP : écoute sur 127.0.0.1, port éphémère.

use std::io;

use tokio::net::TcpListener;
use tokio::task::JoinHandle;

use super::listener::accept_loop;
use super::state::SmtpShared;
use crate::credentials::Credentials;

/// Faux serveur SMTP ; l'écoute s'arrête quand le handle est détruit.
#[derive(Debug)]
pub struct FakeSmtpServer {
    pub(super) port: u16,
    pub(super) shared: SmtpShared,
    task: JoinHandle<()>,
}

impl FakeSmtpServer {
    /// Démarre le serveur sur un port éphémère de la boucle locale
    pub async fn start(credentials: Credentials) -> io::Result<Self> {
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let port = listener.local_addr()?.port();
        let shared = SmtpShared::default();
        let task = tokio::spawn(accept_loop(listener, shared.clone(), credentials));
        Ok(Self { port, shared, task })
    }

    /// Port d'écoute éphémère
    pub fn port(&self) -> u16 {
        self.port
    }

    /// Adresse d'écoute au format 127.0.0.1:port
    pub fn address(&self) -> String {
        format!("127.0.0.1:{}", self.port)
    }

    /// Arrête l'écoute et coupe les connexions en cours
    pub fn shutdown(&self) {
        self.task.abort();
    }
}

impl Drop for FakeSmtpServer {
    /// Arrête la tâche d'écoute à la destruction du handle
    fn drop(&mut self) {
        self.task.abort();
    }
}
