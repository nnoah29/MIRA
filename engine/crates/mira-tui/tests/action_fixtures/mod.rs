//! Fabriques d'états, de commandes et de réponses attendues pour les tests des actions.
#![allow(dead_code, unused_imports)]

mod commands;
mod state;

pub use commands::{
    action_command, action_reply, reload_command, reload_reply, undo_command, undo_reply,
};
pub use state::{ids, state_with_mailboxes, state_with_nonce};
