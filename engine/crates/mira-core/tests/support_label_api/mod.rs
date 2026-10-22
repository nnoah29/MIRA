#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    dead_code,
    unused_imports
)]
//! Support des tests du domaine « local_label » de la façade : appels, fils du miroir, événements.

pub mod calls;
pub mod settings;
pub mod threads;
