//! Délais et plafonds d'un échange avec un service d'IA.

use std::time::Duration;

/// Délais (connexion, premier octet, inactivité) et plafonds de taille d'un échange.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TransportLimits {
    pub connect: Duration,
    pub first_byte: Duration,
    pub idle: Duration,
    pub max_body: usize,
    pub max_stream: usize,
}

impl Default for TransportLimits {
    /// Donne les valeurs fixées par le plan : 10 s, 60 s, 60 s, 2 Mio et 8 Mio.
    fn default() -> Self {
        Self {
            connect: Duration::from_secs(10),
            first_byte: Duration::from_secs(60),
            idle: Duration::from_secs(60),
            max_body: 2 * 1024 * 1024,
            max_stream: 8 * 1024 * 1024,
        }
    }
}
