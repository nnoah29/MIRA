//! Outils partagés des tests IMAP : serveur de test, connexions préparées, courrier d'exemple.
#![allow(dead_code)]

pub mod env;
pub mod logins;
pub mod mail;
pub mod scripted;

use std::future::Future;
use std::time::Duration;

/// Exécute le futur avec une limite de temps pour ne jamais bloquer
pub async fn within<T>(future: impl Future<Output = T>) -> T {
    tokio::time::timeout(Duration::from_secs(20), future)
        .await
        .expect("test timed out")
}
