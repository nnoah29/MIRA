//! Outils des tests de génération : connexions prêtes, requêtes, lecture du flux et attentes bornées.
#![allow(dead_code)]

mod connection;
mod flow;
mod start;

#[allow(unused_imports)]
pub use connection::{add_chat_connection, chat_start, give_consent, make_default, user_request};
#[allow(unused_imports)]
pub use flow::{collect, operation_status, wait_closed, wait_status};
#[allow(unused_imports)]
pub use start::{journal_kinds, loopback_stream, wait_cancelled, wait_first_request};
