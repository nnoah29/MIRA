//! Démon MIRA : héberge le moteur (via `mira-host`) et expose le contrat JSON par le protocole de `mira-protocol`.

pub mod config;
pub mod desktop;
pub mod idle;
mod listen;
mod report;
pub mod run;
pub mod server;
pub mod session;
mod setup;
pub mod shutdown;
mod signals;
pub mod token;
pub mod transport;
pub mod unlock;
pub mod wire;
