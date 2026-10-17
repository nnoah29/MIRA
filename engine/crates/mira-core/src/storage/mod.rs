//! Stockage SQLite d'un profil : seul module qui connaît `rusqlite` et `zstd` (annexe H).

mod database;
mod erasure;
mod errors;
mod introspection;
pub(crate) mod json_columns;
mod migrations;
mod pagination;
pub mod repositories;
mod tx;

pub use database::Database;
pub use migrations::schema_version;
pub use tx::Tx;
