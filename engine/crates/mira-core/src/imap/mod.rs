//! Client IMAP de haut niveau (annexe H) : connexion, boîtes, messages, synchronisation, IDLE.

#![allow(dead_code)]

mod client;
mod connection;
mod errors;
mod operations;
mod parse;
mod protocol;
mod types;

pub use client::ImapClient;
pub use types::{
    FetchedFlags, FetchedHeaders, IdleOutcome, ImapCredential, ImapFlag, ImapLogin, ImapOptions,
    ResyncFrom, SelectedMailbox,
};
