#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    dead_code,
    unused_imports
)]
//! Support des tests de session d'abonné : faux point de jeton, navigateur simulé, jeton d'identité.

pub mod browser;
pub mod connect;
pub mod flow;
pub mod http_read;
pub mod id_token;
pub mod token_server;
