//! Fabriques d'états, de réponses et de commandes attendues pour les tests du lecteur.
#![allow(dead_code, unused_imports)]

mod commands;
mod data;
mod replies;

pub use commands::{body_reply, fetch_body_command, get_body_command, mark_read_command};
pub use data::{attachment, body, list_state, offline_error, thread_view, unread_message};
pub use replies::{attachments_reply, thread_reply};
