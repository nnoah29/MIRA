#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    dead_code,
    unused_imports
)]
//! Support des tests du domaine « rule » de la façade : appels, écritures, règles d'exemple, miroir garni.

pub mod calls;
pub mod files;
pub mod invalid;
pub mod portability;
pub mod refusals;
pub mod samples;
pub mod seed;
pub mod writes;
