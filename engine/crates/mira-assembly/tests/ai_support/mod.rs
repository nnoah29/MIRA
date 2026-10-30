//! Moteur de production assemblé, trousseau en mémoire et faux serveur d'IA pour les tests de bout en bout.
#![allow(dead_code, unused_imports)]

mod fixture;
mod flow;
mod watch;

pub use fixture::{Fixture, KEY, start};
pub use flow::{
    ask_ai, chat, chat_with, connection_ready, consent_test_and_list_models, journal_rows,
};
pub use watch::{answer_text, cancel, collect, events_text, slow_stream, wait_closed};
