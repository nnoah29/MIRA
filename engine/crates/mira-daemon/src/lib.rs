//! Démon MIRA : héberge le moteur (via `mira-host`) et expose le contrat JSON par le protocole de `mira-protocol`.

pub mod server;
pub mod session;
pub mod token;
pub mod transport;
pub mod wire;
