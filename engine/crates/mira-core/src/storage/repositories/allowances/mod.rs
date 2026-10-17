//! Dépôt « allowances » : autorisations de contenu distant (voir docs/specs/J2-stockage.md).

mod model;
mod operations;

pub use model::{RemoteContentAllowance, RemoteContentScope};
