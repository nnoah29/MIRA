#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, dead_code)]
//! Outils des tests de synchronisation : monde de test, faux provider, identifiants, courrier, relevés du miroir.

pub mod actions;
pub mod credentials;
pub mod cutting;
pub mod events;
pub mod inspect;
pub mod mail;
pub mod mirror;
pub mod provider;
pub mod provider_builders;
pub mod reply_rules;
pub mod rules;
pub mod scenarios;
pub mod supervised;
pub mod waiting;
pub mod world;

use std::future::Future;
use std::time::Duration;

/// Exécute le futur avec une limite de temps pour ne jamais bloquer
pub async fn within<T>(future: impl Future<Output = T>) -> T {
    tokio::time::timeout(Duration::from_secs(30), future)
        .await
        .expect("test timed out")
}
