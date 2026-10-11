//! Boîtes : rôles des dossiers (6.8, 10.14).

pub mod common_names;
pub mod role;
pub mod special_use;

pub use role::infer_mailbox_role;
