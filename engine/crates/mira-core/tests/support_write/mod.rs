//! Banc des écritures par la façade : moteur réel, faux serveurs IMAP et SMTP, horloge manuelle, miroir garni.
#![allow(dead_code)]

pub mod build;
pub mod builders;
pub mod compose;
pub mod drafts;
pub mod fixture;
pub mod mailbox_builders;
pub mod oauth;
pub mod oauth_bench;
pub mod requests;
pub mod restart;
pub mod second;
pub mod seed;
pub mod selection;
pub mod server_view;
pub mod servers;
pub mod timing;
pub mod waits;

pub const USER: &str = "alice@folders.test";
pub const SECOND_USER: &str = "carol@other.test";
pub const PASSWORD: &str = "pa55-SECRET-xyz";
pub const START_MILLIS: i64 = 1_700_000_000_000;
