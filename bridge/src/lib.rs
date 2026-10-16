//! Pont FFI de MIRA : expose le moteur à Flutter par des échanges JSON du contrat versionné.
//! Seul `frb_generated` (code généré) contient du `unsafe` ; le reste du crate n'en a pas.

pub mod api;
pub mod envelope;
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod frb_generated;
pub mod original_html;
pub mod runtime;
pub mod session;
