//! Transition pure `(état, message) -> commandes` ; aucune I/O.

pub mod account_switch;
pub mod accounts_reply;
pub mod action_command;
pub mod action_reply;
pub mod actions;
pub mod engine_event;
pub mod event_reaction;
pub mod help;
pub mod intent;
pub mod intent_actions;
pub mod mailboxes_reply;
pub mod mark_read;
pub mod message;
pub mod message_load;
pub mod message_refresh;
pub mod messages_reply;
pub mod move_picker;
pub mod navigation;
pub mod pagination;
pub mod panes;
pub mod popup_intent;
pub mod reader_open;
pub mod reader_reply;
pub mod search;
pub mod search_intent;
pub mod search_reply;
pub mod status_reply;
pub mod sync_status;
pub mod undo;

pub use intent::Intent;
pub use message::{Msg, Reply, update};
pub use search_intent::SearchIntent;
