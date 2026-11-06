//! Écouteur TCP sur 127.0.0.1 uniquement ; l'authentification repose sur le jeton local.

use std::net::SocketAddr;

use tokio::net::TcpListener;

use super::{Accepted, Listener, Trust};

/// Écoute sur la boucle locale ; jamais sur une autre interface.
#[derive(Debug)]
pub struct LoopbackTcpListener {
    listener: TcpListener,
}

impl LoopbackTcpListener {
    /// Lie 127.0.0.1 au port demandé (0 = port libre).
    pub fn bind(port: u16) -> std::io::Result<Self> {
        let std_listener = std::net::TcpListener::bind(("127.0.0.1", port))?;
        std_listener.set_nonblocking(true)?;
        Ok(Self {
            listener: TcpListener::from_std(std_listener)?,
        })
    }

    /// Adresse locale réellement liée.
    pub fn local_addr(&self) -> SocketAddr {
        self.listener
            .local_addr()
            .unwrap_or_else(|_| SocketAddr::from(([127, 0, 0, 1], 0)))
    }

    /// Port réellement lié.
    pub fn local_port(&self) -> u16 {
        self.local_addr().port()
    }
}

impl Listener for LoopbackTcpListener {
    /// Accepte la prochaine connexion venue de la boucle locale ; les autres sont fermées.
    async fn accept(&self) -> std::io::Result<Accepted> {
        loop {
            let (stream, peer) = self.listener.accept().await?;
            if peer.ip().is_loopback() {
                return Ok(Accepted {
                    stream: Box::new(stream),
                    trust: Trust::LocalUser(None),
                });
            }
        }
    }

    /// `tcp:127.0.0.1:<port>`
    fn describe(&self) -> String {
        format!("tcp:{}", self.local_addr())
    }
}
