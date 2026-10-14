//! Transition pure `(état, message) -> commandes` ; aucune I/O.

pub mod account_switch;
pub mod accounts_reply;
pub mod intent;
pub mod mailboxes_reply;
pub mod mark_read;
pub mod message;
pub mod message_load;
pub mod messages_reply;
pub mod navigation;
pub mod pagination;
pub mod panes;
pub mod reader_open;
pub mod reader_reply;

pub use intent::Intent;
pub use message::{Msg, Reply, update};
