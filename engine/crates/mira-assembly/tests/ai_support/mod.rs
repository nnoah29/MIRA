//! Moteur de production assemblé, trousseau en mémoire et faux serveur d'IA pour les tests de bout en bout.
#![allow(dead_code)]

mod fixture;
mod flow;
mod watch;

pub use fixture::{KEY, start};
pub use flow::{ask_ai, chat, connection_ready, consent_test_and_list_models};
pub use watch::{cancel, collect, events_text, slow_stream, wait_closed};
