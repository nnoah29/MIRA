#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    dead_code,
    unused_imports
)]
//! Support des tests de connexion d'IA par navigateur : faux point d'échange, navigateur simulé, moteur prêt.

pub mod browser;
pub mod exchange_server;
pub mod fixture;
