#![allow(dead_code, clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Clients HTTP bruts, écouteur de redirection et scénarios des tests du serveur OAuth.

pub mod exchange;
pub mod flow;
pub mod http;
pub mod loopback;

use std::time::Duration;

pub const LIMIT: Duration = Duration::from_secs(10);
