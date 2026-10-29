#![allow(dead_code, clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Client HTTP brut, analyse des réponses et attente de conditions pour les tests de l'IA.

pub mod client;
pub mod reply;
pub mod wait;

use std::time::Duration;

pub const LIMIT: Duration = Duration::from_secs(10);
