#![allow(dead_code, clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Client SMTP brut et scénarios partagés par les tests du faux serveur SMTP.

pub mod client;
pub mod flow;

use std::time::Duration;

pub const LIMIT: Duration = Duration::from_secs(10);
