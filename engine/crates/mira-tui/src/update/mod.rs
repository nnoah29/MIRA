//! Transition pure `(état, message) -> commandes` ; aucune I/O.

pub mod intent;
pub mod message;
pub mod navigation;
pub mod panes;

pub use intent::Intent;
pub use message::{Msg, Reply, update};
