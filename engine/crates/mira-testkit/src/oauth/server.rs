//! Handle du faux serveur OAuth2 : écoute sur 127.0.0.1, port éphémère.

use std::fmt;
use std::io;

use tokio::net::TcpListener;
use tokio::task::JoinHandle;

use super::context::OAuthContext;
use super::listener::accept_loop;
use super::state::OAuthShared;
use crate::credentials::Credentials;

/// Faux serveur d'autorisation ; l'écoute s'arrête quand le handle est détruit.
pub struct FakeOAuthServer {
    pub(super) port: u16,
    pub(super) context: OAuthContext,
    task: JoinHandle<()>,
}

impl FakeOAuthServer {
    /// Démarre le serveur sur un port éphémère de la boucle locale
    pub async fn start(credentials: Credentials) -> io::Result<Self> {
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let port = listener.local_addr()?.port();
        let shared = OAuthShared::default();
        let context = OAuthContext {
            shared,
            credentials,
        };
        let task = tokio::spawn(accept_loop(listener, context.clone()));
        Ok(Self {
            port,
            context,
            task,
        })
    }

    /// Arrête l'écoute et coupe les connexions en cours
    pub fn shutdown(&self) {
        self.task.abort();
    }
}

impl fmt::Debug for FakeOAuthServer {
    /// Affiche le port seulement, jamais d'identifiants ni de jetons
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("FakeOAuthServer")
            .field("port", &self.port)
            .finish()
    }
}

impl Drop for FakeOAuthServer {
    /// Arrête la tâche d'écoute à la destruction du handle
    fn drop(&mut self) {
        self.task.abort();
    }
}
