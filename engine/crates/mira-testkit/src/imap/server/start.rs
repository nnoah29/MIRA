//! Démarrage du faux serveur.

use std::sync::Arc;

use tokio::net::TcpListener;

use super::accept::accept_loop;
use super::context::ServerContext;
use super::handle::FakeImapServer;
use super::tracker::Tracker;
use crate::credentials::Credentials;
use crate::imap::capabilities::CapabilitySet;
use crate::imap::store::FakeMailStore;

impl FakeImapServer {
    /// Démarre un serveur sur 127.0.0.1 avec un port éphémère
    pub async fn start(
        store: FakeMailStore,
        credentials: Credentials,
        caps: CapabilitySet,
    ) -> std::io::Result<Self> {
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let port = listener.local_addr()?.port();
        let tracker = Arc::new(Tracker::default());
        let context = Arc::new(ServerContext {
            store: store.clone(),
            credentials,
            caps,
        });
        let listener = tokio::spawn(accept_loop(listener, context, Arc::clone(&tracker)));
        Ok(Self {
            port,
            store,
            tracker,
            listener,
        })
    }
}
