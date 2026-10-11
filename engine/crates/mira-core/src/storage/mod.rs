//! Stockage SQLite d'un profil : seul module qui connaît `rusqlite` et `zstd` (annexe H).

mod database;
mod errors;
mod introspection;
#[allow(dead_code)]
mod json_columns;
mod migrations;
pub mod repositories;
mod tx;

pub use database::Database;
pub use migrations::schema_version;
pub use tx::Tx;
