//! Transition pure `(état, message) -> commandes` ; aucune I/O.

pub mod accounts_reply;
pub mod intent;
pub mod mailboxes_reply;
pub mod message;
pub mod navigation;
pub mod panes;

pub use intent::Intent;
pub use message::{Msg, Reply, update};
