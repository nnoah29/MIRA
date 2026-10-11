//! Outils partagés des tests IMAP : client TCP brut, réponses, serveur de test.
#![allow(dead_code)]

pub mod client;
pub mod fixtures;
pub mod reading;
pub mod response;

use std::future::Future;
use std::time::Duration;

/// Exécute le futur avec une limite de temps pour ne jamais bloquer
pub async fn within<T>(future: impl Future<Output = T>) -> T {
    tokio::time::timeout(Duration::from_secs(10), future)
        .await
        .expect("test timed out")
}
