#![allow(
    dead_code,
    unused_imports,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic
)]
//! Support des tests du démon : démon jetable, client brut qui enregistre les octets reçus, moteur de test.

pub mod client;
pub mod daemon;
mod daemon_access;
pub mod engine_config;

pub use client::{Recv, TestClient, hello_with, valid_hello};
pub use daemon::TestDaemon;
pub use engine_config::config;
