//! Transports : un écouteur rend des flux d'octets bidirectionnels et dit à quel point on peut s'y fier.

pub mod port_file;
pub mod tcp;
#[cfg(unix)]
pub mod unix;
#[cfg(unix)]
mod unix_files;

use std::future::Future;

use tokio::io::{AsyncRead, AsyncWrite};

/// Flux d'octets bidirectionnel fiable : tout ce que le protocole demande à un transport.
pub trait ByteStream: AsyncRead + AsyncWrite + Send + Unpin {}

impl<T: AsyncRead + AsyncWrite + Send + Unpin> ByteStream for T {}

/// Confiance que le transport accorde au pair, que la politique d'accès consulte.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Trust {
    LocalUser(Option<u32>),
    Remote,
}

/// Une connexion acceptée avec la confiance du transport.
pub struct Accepted {
    pub stream: Box<dyn ByteStream>,
    pub trust: Trust,
}

impl std::fmt::Debug for Accepted {
    /// N'affiche que la confiance, jamais le flux
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Accepted")
            .field("trust", &self.trust)
            .finish()
    }
}

/// Source de connexions : socket Unix, TCP loopback, plus tard TLS.
pub trait Listener: Send + Sync {
    /// Attend la prochaine connexion digne d'être servie.
    fn accept(&self) -> impl Future<Output = std::io::Result<Accepted>> + Send;

    /// Description lisible de l'adresse d'écoute, sans secret.
    fn describe(&self) -> String;
}
