//! Outils des tests de génération : connexions prêtes, requêtes, lecture du flux et attentes bornées.
#![allow(dead_code)]

mod connection;
mod flow;
mod masking;
mod privacy;
mod start;

#[allow(unused_imports)]
pub use connection::{add_chat_connection, chat_start, give_consent, make_default, user_request};
#[allow(unused_imports)]
pub use flow::{collect, operation_status, wait_closed, wait_status};
#[allow(unused_imports)]
pub use masking::{answer_of, connection_with, run_chat};
#[allow(unused_imports)]
pub use privacy::{allow_kinds, chat_start_with, journal_dump, journal_rows, set_masking};
#[allow(unused_imports)]
pub use start::{journal_kinds, loopback_stream, wait_cancelled, wait_first_request};
