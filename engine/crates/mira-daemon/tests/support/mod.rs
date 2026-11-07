#![allow(
    dead_code,
    unused_imports,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic
)]
//! Support des tests du démon : démon jetable, client brut qui enregistre les octets reçus, moteur de test.

pub mod ai_daemon;
pub mod chat_helpers;
pub mod client;
pub mod daemon;
mod daemon_access;
pub mod engine_config;
pub mod event_helpers;
pub mod locked_store;
pub mod mail_daemon;
pub mod process;
pub mod process_args;
pub mod unlock_harness;

#[allow(clippy::duplicate_mod)]
#[path = "../contract/chat_fixture.rs"]
pub mod chat_fixture;
#[allow(clippy::duplicate_mod)]
#[path = "../contract/mail_fixture.rs"]
pub mod mail_fixture;
#[allow(clippy::duplicate_mod)]
#[path = "../contract/mail_samples.rs"]
pub mod mail_samples;

pub use client::{Recv, TestClient, hello_with, valid_hello};
pub use daemon::TestDaemon;
pub use engine_config::config;
