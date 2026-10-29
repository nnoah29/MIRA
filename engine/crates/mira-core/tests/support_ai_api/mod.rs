#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    dead_code,
    unused_imports
)]
//! Support des tests du domaine « Ai » de la façade : appels, lecture des réponses et fournisseurs de test.

pub mod calls;
pub mod destination;
pub mod flaky_store;
pub mod operations;
pub mod providers;
pub mod session;
